# ZeptoRPC Pragmatic Task Runner

default: check

# Validate workspace formatting and zero-warning linting rules
check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings

# Execute workspace unit and integration test suites
test:
    cargo test --workspace --exclude zepto-ebpf

# Run formal bounded model checking on lock-free data structures
verify-kani:
    cargo kani --dir verification/kani

# Run high-frequency fuzzing target for packet parsing
fuzz-parser RUN_TIME="30s":
    cargo +nightly fuzz run frame_parser -- -max_total_time={{RUN_TIME}}

# Execute end-to-end nanosecond microbenchmarks
bench:
    cargo bench --workspace

# Configure AF_XDP network driver interface and eBPF UMEM rings
setup-af-xdp IFACE="eth0":
    sudo ./scripts/setup_af_xdp.sh {{IFACE}}

# Launch production server binary with native hardware vectorization
run-prod:
    RUSTFLAGS="-C target-cpu=native" cargo run --release --bin zepto-server