use std::{str::FromStr, sync::Arc};

use bencher_json::{
    perf::{JsonPerfData, JsonPerfDatum, JsonPerfDatumKind, JsonPerfKind},
    JsonPerf, JsonPerfQuery,
};
use diesel::{ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl};
use dropshot::{
    endpoint, HttpError, HttpResponseAccepted, HttpResponseHeaders, RequestContext, TypedBody,
};
use uuid::Uuid;

use crate::{
    endpoints::{endpoint::pub_response_accepted, orgs::Resource, Endpoint, Method},
    model::{
        perf::{latency::QueryLatency, resource::QueryResource, throughput::QueryThroughput},
        report::to_date_time,
        user::QueryUser,
    },
    schema,
    util::{
        cors::{get_cors, CorsResponse},
        headers::CorsHeaders,
        map_http_error, Context,
    },
    ApiError,
};

// TODO figure out why this doesn't work as a PUT
#[endpoint {
    method = OPTIONS,
    path =  "/v0/perf",
    tags = ["perf"]
}]
pub async fn options(_rqctx: Arc<RequestContext<Context>>) -> Result<CorsResponse, HttpError> {
    Ok(get_cors::<Context>())
}

#[endpoint {
    method = POST,
    path =  "/v0/perf",
    tags = ["perf"]
}]
pub async fn post(
    rqctx: Arc<RequestContext<Context>>,
    body: TypedBody<JsonPerfQuery>,
) -> Result<HttpResponseHeaders<HttpResponseAccepted<JsonPerf>, CorsHeaders>, HttpError> {
    // This is both a public and auth route
    // Only public projects are allowed to use it as a public route though
    let user_id = Some(QueryUser::auth(&rqctx).await?);
    let endpoint = Endpoint::new(Resource::Perf, Method::Post);

    let context = rqctx.context();
    let json_perf_query = body.into_inner();
    let json = post_inner(user_id, context, json_perf_query)
        .await
        .map_err(|e| endpoint.err(e))?;

    pub_response_accepted!(endpoint, json)
}

async fn post_inner(
    _user_id: Option<i32>,
    context: &Context,
    json_perf_query: JsonPerfQuery,
) -> Result<JsonPerf, ApiError> {
    let JsonPerfQuery {
        branches,
        testbeds,
        benchmarks,
        kind,
        start_time,
        end_time,
    } = json_perf_query;

    // In order to make the type system happy, always query a start and end time.
    // If either is missing then just default to the extremes: zero and max.
    let start_time_nanos = start_time
        .as_ref()
        .map(|t| t.timestamp_nanos())
        .unwrap_or_default();
    let end_time_nanos = end_time
        .as_ref()
        .map(|t| t.timestamp_nanos())
        .unwrap_or(i64::MAX);

    let order_by = (
        schema::version::number,
        schema::report::start_time,
        schema::perf::iteration,
    );

    let context = &mut *context.lock().await;
    let conn = &mut context.db;
    let mut data = Vec::new();
    for branch in &branches {
        for testbed in &testbeds {
            for benchmark in &benchmarks {
                let query = schema::perf::table
                    .left_join(
                        schema::benchmark::table
                            .on(schema::perf::benchmark_id.eq(schema::benchmark::id)),
                    )
                    .filter(schema::benchmark::uuid.eq(benchmark.to_string()))
                    .inner_join(
                        schema::report::table.on(schema::perf::report_id.eq(schema::report::id)),
                    )
                    .filter(schema::report::start_time.ge(start_time_nanos))
                    .filter(schema::report::end_time.le(end_time_nanos))
                    .left_join(
                        schema::testbed::table
                            .on(schema::report::testbed_id.eq(schema::testbed::id)),
                    )
                    .filter(schema::testbed::uuid.eq(testbed.to_string()))
                    .inner_join(
                        schema::version::table
                            .on(schema::report::version_id.eq(schema::version::id)),
                    )
                    .left_join(
                        schema::branch::table.on(schema::version::branch_id.eq(schema::branch::id)),
                    )
                    .filter(schema::branch::uuid.eq(branch.to_string()));

                let query_data: Vec<QueryPerfDatum> = match kind {
                    JsonPerfKind::Latency => query
                        .inner_join(
                            schema::latency::table
                                .on(schema::perf::latency_id.eq(schema::latency::id.nullable())),
                        )
                        .select((
                            schema::perf::uuid,
                            schema::perf::iteration,
                            schema::report::start_time,
                            schema::report::end_time,
                            schema::version::number,
                            schema::version::hash,
                            schema::latency::id,
                            schema::latency::uuid,
                            schema::latency::lower_variance,
                            schema::latency::upper_variance,
                            schema::latency::duration,
                        ))
                        .order(&order_by)
                        .load::<(
                            String,
                            i32,
                            i64,
                            i64,
                            i32,
                            Option<String>,
                            i32,
                            String,
                            i64,
                            i64,
                            i64,
                        )>(conn)
                        .map_err(map_http_error!("Failed to get benchmark data."))?
                        .into_iter()
                        .map(
                            |(
                                uuid,
                                iteration,
                                start_time,
                                end_time,
                                version_number,
                                version_hash,
                                latency_id,
                                latency_uuid,
                                lower_variance,
                                upper_variance,
                                duration,
                            )| {
                                let metrics = QueryPerfMetrics::Latency(QueryLatency {
                                    id: latency_id,
                                    uuid: latency_uuid,
                                    lower_variance,
                                    upper_variance,
                                    duration,
                                });
                                QueryPerfDatum {
                                    uuid,
                                    iteration,
                                    start_time,
                                    end_time,
                                    version_number,
                                    version_hash,
                                    metrics,
                                }
                            },
                        )
                        .collect(),
                    JsonPerfKind::Throughput => {
                        query
                            .inner_join(schema::throughput::table.on(
                                schema::perf::throughput_id.eq(schema::throughput::id.nullable()),
                            ))
                            .select((
                                schema::perf::uuid,
                                schema::perf::iteration,
                                schema::report::start_time,
                                schema::report::end_time,
                                schema::version::number,
                                schema::version::hash,
                                schema::throughput::id,
                                schema::throughput::uuid,
                                schema::throughput::lower_variance,
                                schema::throughput::upper_variance,
                                schema::throughput::events,
                                schema::throughput::unit_time,
                            ))
                            .order(&order_by)
                            .load::<(
                                String,
                                i32,
                                i64,
                                i64,
                                i32,
                                Option<String>,
                                i32,
                                String,
                                f64,
                                f64,
                                f64,
                                i64,
                            )>(conn)
                            .map_err(map_http_error!("Failed to get benchmark data."))?
                            .into_iter()
                            .map(
                                |(
                                    uuid,
                                    iteration,
                                    start_time,
                                    end_time,
                                    version_number,
                                    version_hash,
                                    throughput_id,
                                    throughput_uuid,
                                    lower_variance,
                                    upper_variance,
                                    events,
                                    unit_time,
                                )| {
                                    let metrics = QueryPerfMetrics::Throughput(QueryThroughput {
                                        id: throughput_id,
                                        uuid: throughput_uuid,
                                        lower_variance,
                                        upper_variance,
                                        events,
                                        unit_time,
                                    });
                                    QueryPerfDatum {
                                        uuid,
                                        iteration,
                                        start_time,
                                        end_time,
                                        version_number,
                                        version_hash,
                                        metrics,
                                    }
                                },
                            )
                            .collect()
                    },
                    JsonPerfKind::Compute => query
                        .inner_join(
                            schema::resource::table
                                .on(schema::perf::compute_id.eq(schema::resource::id.nullable())),
                        )
                        .select((
                            schema::perf::uuid,
                            schema::perf::iteration,
                            schema::report::start_time,
                            schema::report::end_time,
                            schema::version::number,
                            schema::version::hash,
                            schema::resource::id,
                            schema::resource::uuid,
                            schema::resource::min,
                            schema::resource::max,
                            schema::resource::avg,
                        ))
                        .order(&order_by)
                        .load::<(
                            String,
                            i32,
                            i64,
                            i64,
                            i32,
                            Option<String>,
                            i32,
                            String,
                            f64,
                            f64,
                            f64,
                        )>(conn)
                        .map_err(map_http_error!("Failed to get benchmark data."))?
                        .into_iter()
                        .map(
                            |(
                                uuid,
                                iteration,
                                start_time,
                                end_time,
                                version_number,
                                version_hash,
                                mma_id,
                                mma_uuid,
                                min,
                                max,
                                avg,
                            )| {
                                let metrics = QueryPerfMetrics::Compute(QueryResource {
                                    id: mma_id,
                                    uuid: mma_uuid,
                                    min,
                                    max,
                                    avg,
                                });
                                QueryPerfDatum {
                                    uuid,
                                    iteration,
                                    start_time,
                                    end_time,
                                    version_number,
                                    version_hash,
                                    metrics,
                                }
                            },
                        )
                        .collect(),
                    JsonPerfKind::Memory => query
                        .inner_join(
                            schema::resource::table
                                .on(schema::perf::memory_id.eq(schema::resource::id.nullable())),
                        )
                        .select((
                            schema::perf::uuid,
                            schema::perf::iteration,
                            schema::report::start_time,
                            schema::report::end_time,
                            schema::version::number,
                            schema::version::hash,
                            schema::resource::id,
                            schema::resource::uuid,
                            schema::resource::min,
                            schema::resource::max,
                            schema::resource::avg,
                        ))
                        .order(&order_by)
                        .load::<(
                            String,
                            i32,
                            i64,
                            i64,
                            i32,
                            Option<String>,
                            i32,
                            String,
                            f64,
                            f64,
                            f64,
                        )>(conn)
                        .map_err(map_http_error!("Failed to get benchmark data."))?
                        .into_iter()
                        .map(
                            |(
                                uuid,
                                iteration,
                                start_time,
                                end_time,
                                version_number,
                                version_hash,
                                mma_id,
                                mma_uuid,
                                min,
                                max,
                                avg,
                            )| {
                                let metrics = QueryPerfMetrics::Memory(QueryResource {
                                    id: mma_id,
                                    uuid: mma_uuid,
                                    min,
                                    max,
                                    avg,
                                });
                                QueryPerfDatum {
                                    uuid,
                                    iteration,
                                    start_time,
                                    end_time,
                                    version_number,
                                    version_hash,
                                    metrics,
                                }
                            },
                        )
                        .collect(),
                    JsonPerfKind::Storage => query
                        .inner_join(
                            schema::resource::table
                                .on(schema::perf::storage_id.eq(schema::resource::id.nullable())),
                        )
                        .select((
                            schema::perf::uuid,
                            schema::perf::iteration,
                            schema::report::start_time,
                            schema::report::end_time,
                            schema::version::number,
                            schema::version::hash,
                            schema::resource::id,
                            schema::resource::uuid,
                            schema::resource::min,
                            schema::resource::max,
                            schema::resource::avg,
                        ))
                        .order(&order_by)
                        .load::<(
                            String,
                            i32,
                            i64,
                            i64,
                            i32,
                            Option<String>,
                            i32,
                            String,
                            f64,
                            f64,
                            f64,
                        )>(conn)
                        .map_err(map_http_error!("Failed to get benchmark data."))?
                        .into_iter()
                        .map(
                            |(
                                uuid,
                                iteration,
                                start_time,
                                end_time,
                                version_number,
                                version_hash,
                                mma_id,
                                mma_uuid,
                                min,
                                max,
                                avg,
                            )| {
                                let metrics = QueryPerfMetrics::Storage(QueryResource {
                                    id: mma_id,
                                    uuid: mma_uuid,
                                    min,
                                    max,
                                    avg,
                                });
                                QueryPerfDatum {
                                    uuid,
                                    iteration,
                                    start_time,
                                    end_time,
                                    version_number,
                                    version_hash,
                                    metrics,
                                }
                            },
                        )
                        .collect(),
                };

                let json_perf_data = into_json(*branch, *testbed, *benchmark, query_data)?;

                data.push(json_perf_data);
            }
        }
    }

    Ok(JsonPerf {
        kind,
        start_time,
        end_time,
        benchmarks: data,
    })
}

fn into_json(
    branch: Uuid,
    testbed: Uuid,
    benchmark: Uuid,
    query_data: Vec<QueryPerfDatum>,
) -> Result<JsonPerfData, HttpError> {
    let mut data = Vec::new();
    for query_datum in query_data {
        data.push(QueryPerfDatum::into_json(query_datum)?)
    }
    Ok(JsonPerfData {
        branch,
        testbed,
        benchmark,
        data,
    })
}

#[derive(Debug)]
pub struct QueryPerfDatum {
    pub uuid: String,
    pub iteration: i32,
    pub start_time: i64,
    pub end_time: i64,
    pub version_number: i32,
    pub version_hash: Option<String>,
    pub metrics: QueryPerfMetrics,
}

impl QueryPerfDatum {
    fn into_json(self) -> Result<JsonPerfDatum, HttpError> {
        let Self {
            uuid,
            iteration,
            start_time,
            end_time,
            version_number,
            version_hash,
            metrics,
        } = self;
        Ok(JsonPerfDatum {
            uuid: Uuid::from_str(&uuid)
                .map_err(map_http_error!("Failed to get benchmark data."))?,
            iteration: iteration as u32,
            start_time: to_date_time(start_time)?,
            end_time: to_date_time(end_time)?,
            version_number: version_number as u32,
            version_hash,
            metrics: QueryPerfMetrics::into_json(metrics)?,
        })
    }
}

#[derive(Debug)]
pub enum QueryPerfMetrics {
    Latency(QueryLatency),
    Throughput(QueryThroughput),
    Compute(QueryResource),
    Memory(QueryResource),
    Storage(QueryResource),
}

impl QueryPerfMetrics {
    fn into_json(self) -> Result<JsonPerfDatumKind, HttpError> {
        Ok(match self {
            Self::Latency(latency) => JsonPerfDatumKind::Latency(QueryLatency::into_json(latency)?),
            Self::Throughput(throughput) => {
                JsonPerfDatumKind::Throughput(QueryThroughput::into_json(throughput)?)
            },
            Self::Compute(resource) => {
                JsonPerfDatumKind::Compute(QueryResource::into_json(resource))
            },
            Self::Memory(resource) => JsonPerfDatumKind::Memory(QueryResource::into_json(resource)),
            Self::Storage(resource) => {
                JsonPerfDatumKind::Storage(QueryResource::into_json(resource))
            },
        })
    }
}
