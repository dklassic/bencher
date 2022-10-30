use std::sync::Arc;

use bencher_json::{JsonBenchmark, ResourceId};
use bencher_rbac::project::Permission;
use diesel::{expression_methods::BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use dropshot::{endpoint, HttpError, Path, RequestContext};
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    endpoints::{
        endpoint::{response_ok, ResponseOk},
        Endpoint, Method,
    },
    error::api_error,
    model::{
        project::{benchmark::QueryBenchmark, QueryProject},
        user::auth::AuthUser,
    },
    schema,
    util::{
        cors::{get_cors, CorsResponse},
        error::into_json,
        Context,
    },
    ApiError,
};

use super::Resource;

const BENCHMARK_RESOURCE: Resource = Resource::Benchmark;

#[derive(Deserialize, JsonSchema)]
pub struct GetDirParams {
    pub project: ResourceId,
}

#[endpoint {
    method = OPTIONS,
    path =  "/v0/projects/{project}/benchmarks",
    tags = ["projects", "benchmarks"]
}]
pub async fn dir_options(
    _rqctx: Arc<RequestContext<Context>>,
    _path_params: Path<GetDirParams>,
) -> Result<CorsResponse, HttpError> {
    Ok(get_cors::<Context>())
}

#[endpoint {
    method = GET,
    path =  "/v0/projects/{project}/benchmarks",
    tags = ["projects", "benchmarks"]
}]
pub async fn get_ls(
    rqctx: Arc<RequestContext<Context>>,
    path_params: Path<GetDirParams>,
) -> Result<ResponseOk<Vec<JsonBenchmark>>, HttpError> {
    let auth_user = AuthUser::new(&rqctx).await?;
    let endpoint = Endpoint::new(BENCHMARK_RESOURCE, Method::GetLs);

    let json = get_ls_inner(
        rqctx.context(),
        &auth_user,
        path_params.into_inner(),
        endpoint,
    )
    .await
    .map_err(|e| endpoint.err(e))?;

    response_ok!(endpoint, json)
}

async fn get_ls_inner(
    context: &Context,
    auth_user: &AuthUser,
    path_params: GetDirParams,
    endpoint: Endpoint,
) -> Result<Vec<JsonBenchmark>, ApiError> {
    let api_context = &mut *context.lock().await;
    let query_project = QueryProject::is_allowed_resource_id(
        api_context,
        &path_params.project,
        auth_user,
        Permission::View,
    )?;
    let conn = &mut api_context.database;

    Ok(schema::benchmark::table
        .filter(schema::benchmark::project_id.eq(&query_project.id))
        .order(schema::benchmark::name)
        .load::<QueryBenchmark>(conn)
        .map_err(api_error!())?
        .into_iter()
        .filter_map(into_json!(endpoint, conn))
        .collect())
}

#[derive(Deserialize, JsonSchema)]
pub struct GetOneParams {
    pub project: ResourceId,
    pub benchmark: Uuid,
}

#[endpoint {
    method = OPTIONS,
    path =  "/v0/projects/{project}/benchmarks/{benchmark}",
    tags = ["projects", "benchmarks"]
}]
pub async fn one_options(
    _rqctx: Arc<RequestContext<Context>>,
    _path_params: Path<GetOneParams>,
) -> Result<CorsResponse, HttpError> {
    Ok(get_cors::<Context>())
}

#[endpoint {
    method = GET,
    path =  "/v0/projects/{project}/benchmarks/{benchmark}",
    tags = ["projects", "benchmarks"]
}]
pub async fn get_one(
    rqctx: Arc<RequestContext<Context>>,
    path_params: Path<GetOneParams>,
) -> Result<ResponseOk<JsonBenchmark>, HttpError> {
    let auth_user = AuthUser::new(&rqctx).await?;
    let endpoint = Endpoint::new(BENCHMARK_RESOURCE, Method::GetOne);

    let json = get_one_inner(rqctx.context(), path_params.into_inner(), &auth_user)
        .await
        .map_err(|e| endpoint.err(e))?;

    response_ok!(endpoint, json)
}

async fn get_one_inner(
    context: &Context,
    path_params: GetOneParams,
    auth_user: &AuthUser,
) -> Result<JsonBenchmark, ApiError> {
    let api_context = &mut *context.lock().await;
    let query_project = QueryProject::is_allowed_resource_id(
        api_context,
        &path_params.project,
        auth_user,
        Permission::View,
    )?;
    let conn = &mut api_context.database;

    schema::benchmark::table
        .filter(
            schema::benchmark::project_id
                .eq(query_project.id)
                .and(schema::benchmark::uuid.eq(path_params.benchmark.to_string())),
        )
        .first::<QueryBenchmark>(conn)
        .map_err(api_error!())?
        .into_json(conn)
}
