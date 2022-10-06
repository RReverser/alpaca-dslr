#[path = "../generator/AlpacaDeviceAPI_v1.rs"]
pub mod api;

fn main() -> std::io::Result<()> {
    api::main()
}
