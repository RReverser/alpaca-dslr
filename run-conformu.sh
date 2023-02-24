RUST_LOG=debug cargo run &
RUST=$!
~/conformu/conformu --commandline
kill $RUST
