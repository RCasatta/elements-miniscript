#!/bin/sh

set -ex

FEATURES="compiler serde rand base64 simplicity"

cargo --version
rustc --version

# Pin dependencies required to build with Rust 1.58
if cargo --version | grep "1\.58"; then
    cargo update -p byteorder --precise 1.4.3
    cargo update -p cc --precise 1.0.94
    cargo update -p ppv-lite86 --precise 0.2.17
fi

# Format if told to
if [ "$DO_FMT" = true ]
then
    rustup component add rustfmt
    cargo fmt -- --check
fi

# Test bitcoind integration tests if told to (this only works with the stable toolchain)
if [ "$DO_BITCOIND_TESTS" = true ]; then

    BITCOIND_EXE_DEFAULT="$(git rev-parse --show-toplevel)/bitcoind-tests/bin/bitcoind"
    ELEMENTSD_EXE_DEFAULT="$(git rev-parse --show-toplevel)/bitcoind-tests/bin/elementsd"

    cd bitcoind-tests

    BITCOIND_EXE=${BITCOIND_EXE:=${BITCOIND_EXE_DEFAULT}} \
    ELEMENTSD_EXE=${ELEMENTSD_EXE:=${ELEMENTSD_EXE_DEFAULT}} \
    cargo test --verbose

    # Exit integration tests, do not run other tests.
    exit 0
fi

# Defaults / sanity checks
cargo test

if [ "$DO_FEATURE_MATRIX" = true ]
then
    # All features
    cargo test --features="$FEATURES"

    # Single features
    for feature in ${FEATURES}
    do
        cargo test --features="$feature"
    done

    # Run all the examples
    cargo build --examples
    cargo run --example htlc --features=compiler
    cargo run --example parse
    cargo run --example sign_multisig
    cargo run --example verify_tx > /dev/null
    cargo run --example xpub_descriptors
    cargo run --example taproot --features=compiler
    cargo run --example psbt_sign_finalize --features=base64
fi

# Bench if told to (this only works with the nightly toolchain)
if [ "$DO_BENCH" = true ]
then
    RUSTFLAGS=--cfg=miniscript_bench cargo bench --features="compiler"
fi

# Build the docs if told to (this only works with the nightly toolchain)
if [ "$DO_DOCS" = true ]; then
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly rustdoc --features="$FEATURES" -- -D rustdoc::broken-intra-doc-links
fi

exit 0
