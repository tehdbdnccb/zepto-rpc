//! SIMD-accelerated hardware decoding abstractions for transaction signatures and account keys.

pub mod avx512_base58;

pub use avx512_base58::Avx512Base58Decoder;