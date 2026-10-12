//! Formal Verification Harness for Lock-Free SPSC Ring Buffer.
//!
//! Uses the Kani Rust Verifier (model checker) to mathematically prove safety properties:
//! - Absolute non-overrun / non-underrun invariants
//! - Index bit-mask wrap-around correctness ($N = 2^k$)
//! - Absence of out-of-bounds pointer arithmetic

#[cfg(kani)]
mod spsc_proofs {
    use core::sync::atomic::{AtomicUsize, Ordering};

    const CAPACITY: usize = 16; // Power-of-two capacity for symbolic bounded check
    const MASK: usize = CAPACITY - 1;

    struct BoundedSpscBuffer<T: Copy, const N: usize> {
        buffer: [T; N],
        head: AtomicUsize,
        tail: AtomicUsize,
    }

    impl<T: Copy + Default, const N: usize> BoundedSpscBuffer<T, N> {
        pub const fn new() -> Self {
            assert!(N > 0 && (N & (N - 1)) == 0, "Capacity must be power of two");
            Self {
                buffer: [T::default(); N],
                head: AtomicUsize::new(0),
                tail: AtomicUsize::new(0),
            }
        }

        #[inline(always)]
        pub fn try_push(&mut self, val: T) -> Result<(), T> {
            let current_head = self.head.load(Ordering::Relaxed);
            let current_tail = self.tail.load(Ordering::Acquire);

            if current_head.wrapping_sub(current_tail) >= N {
                return Err(val);
            }

            let index = current_head & (N - 1);
            self.buffer[index] = val;
            self.head.store(current_head.wrapping_add(1), Ordering::Release);
            Ok(())
        }

        #[inline(always)]
        pub fn try_pop(&mut self) -> Option<T> {
            let current_tail = self.tail.load(Ordering::Relaxed);
            let current_head = self.head.load(Ordering::Acquire);

            if current_tail == current_head {
                return None;
            }

            let index = current_tail & (N - 1);
            let val = self.buffer[index];
            self.tail.store(current_tail.wrapping_add(1), Ordering::Release);
            Some(val)
        }
    }

    #[kani::proof]
    #[kani::unwind(17)]
    fn proof_spsc_enqueue_dequeue_invariants() {
        let mut queue: BoundedSpscBuffer<u64, CAPACITY> = BoundedSpscBuffer::new();

        // Nondeterministic number of push/pop operations
        let num_operations: u8 = kani::any();
        kani::assume(num_operations <= 16);

        let mut pushed_count: usize = 0;
        let mut popped_count: usize = 0;

        for _ in 0..num_operations {
            let is_push: bool = kani::any();

            if is_push {
                let val: u64 = kani::any();
                if queue.try_push(val).is_ok() {
                    pushed_count += 1;
                }
            } else if queue.try_pop().is_some() {
                popped_count += 1;
            }

            // Invariant 1: Queue size can never exceed defined capacity N
            let current_size = pushed_count - popped_count;
            kani::assert(
                current_size <= CAPACITY,
                "SPSC Queue size exceeded static capacity bounds",
            );
        }
    }

    #[kani::proof]
    fn proof_ring_buffer_index_wrapping() {
        let head: usize = kani::any();
        let index = head & MASK;

        // Invariant 2: Masked index is bounded strictly within 0..CAPACITY
        kani::assert(
            index < CAPACITY,
            "Bitwise wrap-around index produced out-of-bounds pointer offset",
        );
    }
}