use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

#[path = "../generator/AlpacaDeviceAPI_v1.rs"]
pub mod api;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .finish()
        .init();

    Ok(api::main()?)
}
