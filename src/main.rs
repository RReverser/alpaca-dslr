use actix_web::{App, HttpServer};
use ascom_alpaca_rs::api::{Camera, Device};
use ascom_alpaca_rs::{ASCOMResult, DevicesBuilder, OpaqueResponse};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

struct MyCamera;

impl Device for MyCamera {
    fn ty(&self) -> &'static str {
        <MyCamera as Camera>::TYPE
    }

    fn handle_action(
        &mut self,
        is_mut: bool,
        action: &str,
        params: &str,
    ) -> ASCOMResult<OpaqueResponse> {
        Camera::handle_action_impl(self, is_mut, action, params)
    }
}

impl Camera for MyCamera {}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .finish()
        .init();

    let devices = DevicesBuilder::new().with(MyCamera).finish();

    HttpServer::new(move || {
        App::new()
            .wrap(tracing_actix_web::TracingLogger::default())
            .service(devices.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
