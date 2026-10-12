#![no_main]

use libfuzzer_sys::fuzz_target;
use zepto_core::frame::ZeptoFrameView;

fuzz_target!(|data: &[u8]| {
    // Attempt zero-copy frame parsing from random wire byte sequences
    if let Ok(frame) = ZeptoFrameView::parse(data) {
        // Assert invariants on successfully parsed headers
        let magic = frame.magic();
        let payload_len = frame.payload_length() as usize;

        // Parsed payload slice length must never exceed raw buffer length
        assert!(
            payload_len <= data.len(),
            "Parsed payload length exceeds source buffer length"
        );

        // Header validation integrity assertions
        let _ = frame.sequence_id();
        let _ = frame.timestamp_ns();
        let _ = frame.checksum();
    }
});