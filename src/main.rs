use actix_web::{App, HttpServer};
use api::{Camera, Device, DevicesBuilder, DomainRootSpanBuilder};

use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

#[path = "../generator/AlpacaDeviceAPI_v1.rs"]
pub mod api;

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
    ) -> api::ASCOMResult<serde_json::Value> {
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
            .wrap(tracing_actix_web::TracingLogger::<DomainRootSpanBuilder>::new())
            .service(devices.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
