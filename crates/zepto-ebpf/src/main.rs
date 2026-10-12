//! Kernel-bypass eBPF/XDP ingress packet driver.
//!
//! Attaches to NIC RX queues to inspect raw ethernet frames in kernel space.
//! Valid transactions are redirected directly to AF_XDP socket maps (`XSKMAP`)
//! in zero-copy UMEM shared memory, bypassing the standard Linux networking stack.

#![no_std]
#![no_main]

use aya_ebpf::{
    annotations::xdp,
    bindings::xdp_action,
    macros::map,
    maps::XskMap,
    programs::XdpContext,
};
use core::slice;

/// BPF map binding AF_XDP socket descriptors to network hardware RX ring channels.
#[map(name = "ZEPTO_XSKMAP")]
static ZEPTO_XSKMAP: XskMap = XskMap::with_max_entries(64, 0);

/// Ethernet header parsing constants
const ETH_HLEN: usize = 14;
const IP_HLEN: usize = 20;
const UDP_HLEN: usize = 8;
const MIN_TX_PAYLOAD_SIZE: usize = 64;

#[xdp]
pub fn zepto_ingress_filter(ctx: XdpContext) -> u32 {
    match unsafe { try_zepto_ingress_filter(&ctx) } {
        Ok(action) => action,
        Err(_) => xdp_action::XDP_PASS, // Default fallback to standard kernel network stack
    }
}

#[inline(always)]
unsafe fn try_zepto_ingress_filter(ctx: &XdpContext) -> Result<u32, ()> {
    let start = ctx.data() as *const u8;
    let end = ctx.data_end() as *const u8;

    let len = end.offset_from(start) as usize;

    // Fast-path frame validation: Minimum UDP packet boundary
    if len < ETH_HLEN + IP_HLEN + UDP_HLEN + MIN_TX_PAYLOAD_SIZE {
        return Ok(xdp_action::XDP_PASS);
    }

    let frame = slice::from_raw_parts(start, len);

    // Verify IPv4 EtherType (0x0800)
    if frame[12] != 0x08 || frame[13] != 0x00 {
        return Ok(xdp_action::XDP_PASS);
    }

    // Verify IP Protocol is UDP (17)
    if frame[ETH_HLEN + 9] != 17 {
        return Ok(xdp_action::XDP_PASS);
    }

    // Redirect packet payload directly into AF_XDP socket ring zero-copy memory
    let queue_id = ctx.rx_queue_index();
    if ZEPTO_XSKMAP.redirect(queue_id, 0).is_ok() {
        Ok(xdp_action::XDP_REDIRECT)
    } else {
        Ok(xdp_action::XDP_PASS)
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}