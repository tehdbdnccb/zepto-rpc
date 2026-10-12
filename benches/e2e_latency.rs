//! End-to-End Ingress Nanosecond Latency Benchmark.
//!
//! Measures full transaction ingestion loop latency across:
//! - Stack frame validation & parsing
//! - Thermodynamic payload entropy drop scoring (<150ns target)
//! - Hawkes self-exciting point process intensity evaluation
//! - Multi-path speculative UDP packet dispatch

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::net::SocketAddr;
use std::sync::Arc;

use zepto_core::frame::ZeptoFrameBuilder;
use zepto_net::leader_schedule::{LeaderScheduleTable, LeaderTarget};
use zepto_net::udp::{MultiPathUdpMultiplexer, UdpConfig};
use zepto_quant::entropy::{EntropyConfig, ThermodynamicEntropyFilter};
use zepto_quant::hawkes::{HawkesConfig, HawkesProcessModel};

/// Micro-benchmark harness simulating realistic RPC ingress workloads.
fn bench_e2e_ingress_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("e2e_ingress_latency");

    // Setup network targets & leader table
    let leader_table = Arc::new(LeaderScheduleTable::default());
    let tpu_addr: SocketAddr = "127.0.0.1:8001".parse().unwrap();
    let forward_addr: SocketAddr = "127.0.0.1:8002".parse().unwrap();

    let targets = vec![
        LeaderTarget {
            slot: 1,
            identity: solana_sdk::pubkey::Pubkey::new_unique(),
            tpu_addr,
            tpu_forward_addr: forward_addr,
        },
        LeaderTarget {
            slot: 2,
            identity: solana_sdk::pubkey::Pubkey::new_unique(),
            tpu_addr,
            tpu_forward_addr: forward_addr,
        },
    ];
    leader_table.update_schedule(&targets).unwrap();

    let udp_mux = Arc::new(
        MultiPathUdpMultiplexer::bind("0.0.0.0:0".parse().unwrap(), UdpConfig::default()).unwrap(),
    );
    let entropy_filter = ThermodynamicEntropyFilter::new(EntropyConfig::default());
    let mut hawkes_model = HawkesProcessModel::new(HawkesConfig::default());

    // Construct valid wire frame payload
    let mut raw_buffer = [0u8; 512];
    let payload = [0xAAu8; 128];
    let frame_len = ZeptoFrameBuilder::new(&mut raw_buffer)
        .sequence_id(1001)
        .timestamp_ns(1_700_000_000_000_000_000)
        .write_payload(&payload)
        .unwrap();

    let valid_frame = &raw_buffer[..frame_len];

    // Benchmark 1: Sub-component Thermodynamic Entropy Filter (<150ns target)
    group.throughput(Throughput::Bytes(valid_frame.len() as u64));
    group.bench_function("thermodynamic_entropy_scorer", |b| {
        b.iter(|| {
            let res = entropy_filter.calculate_entropy_q16(black_box(valid_frame));
            black_box(res).unwrap();
        });
    });

    // Benchmark 2: Hawkes Self-Exciting Intensity Point Process
    group.bench_function("hawkes_intensity_update", |b| {
        let mut timestamp = 1_000_000u64;
        b.iter(|| {
            timestamp += 100;
            let intensity = hawkes_model.observe_arrival(black_box(timestamp));
            black_box(intensity);
        });
    });

    // Benchmark 3: Full Wire Ingress & Fanout Loop across varying payload sizes
    for payload_size in [64, 256, 1024].iter() {
        let mut buf = vec![0u8; *payload_size];
        let len = ZeptoFrameBuilder::new(&mut buf)
            .sequence_id(42)
            .timestamp_ns(1_700_000_000_000_000_000)
            .write_payload(&vec![0x42; *payload_size - 32])
            .unwrap();

        let frame = &buf[..len];

        group.throughput(Throughput::Bytes(frame.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("full_ingress_fanout", payload_size),
            frame,
            |b, frame_data| {
                b.iter(|| {
                    let _ = entropy_filter.validate_payload(black_box(frame_data));
                    let leaders = leader_table.get_upcoming_leaders();
                    let sent = udp_mux.speculative_fanout(&leaders, black_box(frame_data));
                    black_box(sent).ok();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_e2e_ingress_pipeline);
criterion_main!(benches);