cargo build
RUST_LOG=ascom_alpaca_rs=debug,alpaca_dslr=debug cargo run &
RUST=$!
~/conformu/conformu --commandline
kill $RUST
