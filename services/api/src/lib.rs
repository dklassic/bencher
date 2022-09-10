#[macro_use]
extern crate diesel;

pub mod endpoints;
pub mod error;
pub mod model;
pub mod schema;
pub mod util;

pub use error::ApiError;

pub enum Endpoint {
    AuthConfirm,
    AuthInvite,
    AuthLogin,
    AuthSignup,
    UserToken,
    Benchmark,
    Branch,
    Perf,
    Ping,
    Project,
    Report,
    Testbed,
    Threshold,
}
