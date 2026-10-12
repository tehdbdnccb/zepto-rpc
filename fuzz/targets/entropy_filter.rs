#![no_main]

use libfuzzer_sys::fuzz_target;
use zepto_quant::entropy::{EntropyConfig, ThermodynamicEntropyFilter};

fuzz_target!(|data: &[u8]| {
    let filter = ThermodynamicEntropyFilter::new(EntropyConfig::default());

    // Execute entropy calculation against arbitrary byte buffers
    if let Ok(entropy) = filter.calculate_entropy_q16(data) {
        // Fixed-point Shannon entropy upper bound validation (8 bits/byte == 524,288 Q16)
        assert!(
            entropy <= 524_288,
            "Calculated entropy exceeded theoretical 8-bit maximum bound: {}",
            entropy
        );
    }

    // Ensure payload validation route never crashes on arbitrary binary frames
    let _ = filter.validate_payload(data);
});