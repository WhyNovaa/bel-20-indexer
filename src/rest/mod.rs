use super::*;

use axum::{
    http::Uri,
    response::{sse::Event, Sse},
    routing::post,
};
use futures::Stream;
use nintypes::common::inscriptions::Outpoint;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::compression::CompressionLayer;
use validator::Validate;

mod address;
mod history;
mod holders;
mod info;
mod tokens;
pub mod types;
mod utils;

type ApiResult<T> = core::result::Result<T, Response<String>>;
const INTERNAL: &str = "Internal server error";
const BAD_REQUEST: &str = "Bad request";
const BAD_PARAMS: &str = "Invalid request params";
const NOT_FOUND: &str = "Not found";

pub fn get_router(server: Arc<Server>) -> Router {
    Router::new()
        .with_state(server)
        .layer(CompressionLayer::new())
}
