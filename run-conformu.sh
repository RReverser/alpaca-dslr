cargo build
RUST_LOG=ascom_alpaca_rs=debug,alpaca_dslr=debug cargo run &
RUST=$!
~/conformu/conformu --commandline --settings $PWD/conform.settings.json
kill $RUST
