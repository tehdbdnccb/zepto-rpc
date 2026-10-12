//! Lock-Free Single-Producer Single-Consumer (SPSC) Ring Buffer Throughput Benchmark.
//!
//! Tests million-operations-per-second (MOPS) throughput limits and memory barrier overhead.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

const QUEUE_CAPACITY: usize = 65536;
const QUEUE_MASK: usize = QUEUE_CAPACITY - 1;

struct SpscQueue<T, const N: usize> {
    buffer: Vec<T>,
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl<T: Copy + Default, const N: usize> SpscQueue<T, N> {
    pub fn new() -> Self {
        let mut buffer = Vec::with_capacity(N);
        buffer.resize_with(N, T::default);
        Self {
            buffer,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    pub fn push(&mut self, item: T) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        if head.wrapping_sub(tail) >= N {
            return false;
        }

        self.buffer[head & (N - 1)] = item;
        self.head.store(head.wrapping_add(1), Ordering::Release);
        true
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        if tail == head {
            return None;
        }

        let item = self.buffer[tail & (N - 1)];
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(item)
    }
}

fn bench_spsc_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("spsc_ring_buffer");
    let batch_size = 1_000_000u64;

    group.throughput(Throughput::Elements(batch_size));

    group.bench_function("spsc_enqueue_dequeue_throughput", |b| {
        b.iter(|| {
            let mut queue = SpscQueue::<u64, QUEUE_CAPACITY>::new();
            for i in 0..batch_size {
                while !queue.push(black_box(i)) {
                    std::hint::spin_loop();
                }
                while queue.pop().is_none() {
                    std::hint::spin_loop();
                }
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_spsc_throughput);
criterion_main!(benches);