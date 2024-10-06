# exit when any command fails
set -e

# build the Rust
RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals -Clink-arg=--max-memory=4294967296 -Clink-arg=--export=__stack_pointer' \
    cargo +nightly build --target wasm32-unknown-unknown -Z build-std=std,panic_abort --release

cp target/wasm32-unknown-unknown/release/ld52.wasm web_build/ld52.wasm
cp -R assets web_build
