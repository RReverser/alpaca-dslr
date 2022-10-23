use ascom_alpaca_rs::api::{Camera, Device};
use ascom_alpaca_rs::{ASCOMParams, ASCOMResult, DevicesBuilder, OpaqueResponse};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

struct MyCamera;

impl Device for MyCamera {
    fn ty(&self) -> &'static str {
        <dyn Camera>::TYPE
    }

    fn handle_action(
        &mut self,
        is_mut: bool,
        action: &str,
        params: ASCOMParams,
    ) -> ASCOMResult<OpaqueResponse> {
        <dyn Camera>::handle_action_impl(self, is_mut, action, params)
    }
}

impl Camera for MyCamera {}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .finish()
        .init();

    let devices = DevicesBuilder::new().with(MyCamera).finish();

    // run our app with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr)
        .serve(
            devices
                .into_router()
                .layer(TraceLayer::new_for_http())
                .into_make_service(),
        )
        .await
        .unwrap();
}
