//! Toolkit for building your own bootloader, tailored to your needs.
#![no_std]

use core::num::NonZeroU16;

pub mod strategies;

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod mock;

#[derive(Debug)]
pub struct Error;

/// Representation of a concrete device with image slots, supporting copying of pages.
#[allow(async_fn_in_trait)]
pub trait Device {
    /// Copy a page from one memory to another.
    async fn copy(&mut self, operation: CopyOperation) -> Result<(), Error>;

    /// Boot a specific memory slot.
    fn boot(slot: Slot) -> !;

    /// All image slots should have the same memory size.
    /// Note that these are `Page` in the bootloader sense, which is decoupled from the underlying memory storage.
    fn page_count(&self) -> NonZeroU16;
}

pub trait DeviceWithScratch: Device {
    /// Number of pages available in the scratch memory.
    fn scratch_page_count(&self) -> NonZeroU16;

    fn get_scratch(&self) -> Slot;
}

pub trait DeviceWithPrimarySlot: Device {
    fn get_primary(&self) -> Slot;
}

/// Image slot with regards to the bootloader.
///
/// Memory layout describes in which memory and at what location each slot resides.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Slot(pub(crate) u8);

/// Page number with regards to the bootloader.
///
/// If the underlying memory has a disparate page size,
/// largest page size is chosen and used for this.
///
/// The page size must be a multiple of all the underlying page sizes.
///
/// For example: with a 1K page size for primary memory and 4K page size for secondary memory,
/// `Page(0)` is 4K large and covers 4 physical pages in primary memory.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Page(pub(crate) u16);

/// Step number of a specific strategy that has to be or has been executed.
///
/// What operation this step entails can be extracted from the strategy.
/// Step numbers are strictly monotonical.
/// Every step can be interrupted at any time, and after a step has been executed this has to be recorded in the persistant state.
/// If the step is executed, but not yet recorded in the persistant state, it must be valid to execute the step again.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Step(pub(crate) u16);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MemoryLocation {
    slot: Slot,
    page: Page,
}

/// Perform an erase of `to` (if necessary) and copy `from` to `to`, leaving `from` intact.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct CopyOperation {
    pub from: MemoryLocation,
    pub to: MemoryLocation,
}
