cargo build
RUST_LOG=ascom_alpaca=debug,alpaca_dslr=debug,gphoto2=debug,gphoto2::gp_port_vusb_find_device_lib=warn cargo run &
RUST=$!
~/conformu/conformu --commandline --settings $PWD/conform.settings.json
kill $RUST
