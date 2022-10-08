use actix_web::{App, HttpServer};
use api::DomainRootSpanBuilder;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

#[path = "../generator/AlpacaDeviceAPI_v1.rs"]
pub mod api;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .finish()
        .init();

    HttpServer::new(|| {
        App::new()
            .wrap(tracing_actix_web::TracingLogger::<DomainRootSpanBuilder>::new())
            .service(api::service())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
