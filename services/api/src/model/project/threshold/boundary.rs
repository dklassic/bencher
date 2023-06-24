use std::str::FromStr;

use bencher_json::{
    project::{benchmark::JsonBenchmarkMetric, boundary::JsonBoundary},
    JsonBenchmark,
};
use diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl};
use uuid::Uuid;

use super::QueryThreshold;
use crate::{
    context::DbConnection,
    error::api_error,
    model::project::{
        benchmark::QueryBenchmark, metric::QueryMetric, perf::QueryPerf, report::QueryReport,
    },
    schema,
    schema::boundary as boundary_table,
    util::query::{fn_get, fn_get_id},
    ApiError,
};

#[derive(Queryable)]
pub struct QueryBoundary {
    pub id: i32,
    pub uuid: String,
    pub perf_id: i32,
    pub threshold_id: i32,
    pub statistic_id: i32,
    pub left_side: Option<f64>,
    pub right_side: Option<f64>,
}

impl QueryBoundary {
    fn_get!(boundary);
    fn_get_id!(boundary);

    pub fn get_uuid(conn: &mut DbConnection, id: i32) -> Result<Uuid, ApiError> {
        let uuid: String = schema::alert::table
            .filter(schema::alert::id.eq(id))
            .select(schema::alert::uuid)
            .first(conn)
            .map_err(api_error!())?;
        Uuid::from_str(&uuid).map_err(api_error!())
    }

    pub fn into_json(self, conn: &mut DbConnection) -> Result<JsonBoundary, ApiError> {
        let Self {
            perf_id,
            threshold_id,
            statistic_id,
            left_side,
            right_side,
            ..
        } = self;

        let perf = QueryPerf::get(conn, perf_id)?;
        let mut threshold = QueryThreshold::get(conn, threshold_id)?;
        // IMPORTANT: Set the statistic ID to the one from the boundary, and not the current value!
        threshold.statistic_id = statistic_id;

        Ok(JsonBoundary {
            report: QueryReport::get_uuid(conn, perf.report_id)?,
            iteration: perf.iteration as u32,
            benchmark: get_benchmark_metric(
                conn,
                perf.id,
                threshold.metric_kind_id,
                perf.benchmark_id,
            )?,
            threshold: threshold.into_json(conn)?,
            left_side: left_side.map(Into::into),
            right_side: right_side.map(Into::into),
        })
    }
}

fn get_benchmark_metric(
    conn: &mut DbConnection,
    perf_id: i32,
    metric_kind_id: i32,
    benchmark_id: i32,
) -> Result<JsonBenchmarkMetric, ApiError> {
    let json_benchmark = schema::benchmark::table
        .filter(schema::benchmark::id.eq(benchmark_id))
        .first::<QueryBenchmark>(conn)
        .map_err(api_error!())?
        .into_json(conn)?;
    let JsonBenchmark {
        uuid,
        project,
        name,
    } = json_benchmark;

    let metric = schema::metric::table
        .filter(schema::metric::perf_id.eq(perf_id))
        .filter(schema::metric::metric_kind_id.eq(metric_kind_id))
        .first::<QueryMetric>(conn)
        .map_err(api_error!())?
        .into_json();

    Ok(JsonBenchmarkMetric {
        uuid,
        project,
        name,
        metric,
    })
}

#[derive(Insertable)]
#[diesel(table_name = boundary_table)]
pub struct InsertBoundary {
    pub uuid: String,
    pub perf_id: i32,
    pub threshold_id: i32,
    pub statistic_id: i32,
    pub left_side: Option<f64>,
    pub right_side: Option<f64>,
}