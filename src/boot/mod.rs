#[cfg(feature = "cortex_m")]
pub mod cortex_m;

/// Bootload mechanism that at the least jumps to the address as defined by an image slot.
///
/// Optionally could drop TrustZone privileges or mask memory access.
pub trait Boot {
    /// Address to the start of the image slot body, typically pointing to the vector table.
    ///
    /// # Safety
    /// Ensure that the address range pointed too is actually a valid vector table in the intended image.
    /// If not the behaviour is undefined.
    unsafe fn boot(addr: *const u32) -> !;
}
