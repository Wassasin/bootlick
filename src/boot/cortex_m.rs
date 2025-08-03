use crate::boot::Boot;

/// Simple bootload mechanism for Cortex-M without support for TrustZone.
pub struct SimpleCortexM;

impl Boot for SimpleCortexM {
    unsafe fn boot(addr: *const u32) -> ! {
        unsafe { cortex_m::asm::bootload(addr) }
    }
}
