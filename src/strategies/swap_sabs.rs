//! Strategy to swap two slots using 'S <- A <- B <- S', leaving both intact after finalizing.
//!
//! In other bootloaders also called 'swap scratch'.
//! It employs a 'scratch' partition as a temporary buffer for one of the pages.
//! This is an especially useful strategy if a memory type is available that is very wear resistant, like FRAM.
//!
//! The primary (A) slot page is first copied over to the scratch memory (S), before writing the secondary (B) slot page to the primary (A) slot page.
//! Finally the scratch (S) memory page is written to the secondary (B) memory page.
//!
//! This results in the primary and secondary slots enduring a single erasure on every page for this strategy, whilst the scratch page endures `N` erasures, where `N` is the number of pages.
//!
//! **TODO** What if the scratch memory is several pages big?

use core::num::NonZeroU16;

use crate::{CopyOperation, MemoryLocation, Page, Slot, Step, strategies::Strategy};

pub struct SwapSABS {
    num_pages: NonZeroU16,
    scratch_pages: NonZeroU16,
    slot_primary: Slot,
    slot_secondary: Slot,
    slot_scratch: Slot,
}

/// Logical phases for the strategy to execute, to decouple raw steps from behaviour in a logical manner.
#[derive(Debug)]
enum Phase {
    ToScratch { start: Page },
    ToPrimary { start: Page },
    ToSecondary { start: Page },
}

impl Phase {
    pub const fn from_step(step: Step, scratch_pages: NonZeroU16) -> Phase {
        let start = Page((step.0 / 3) * scratch_pages.get());
        match step.0 % 3 {
            0 => Phase::ToScratch { start },
            1 => Phase::ToPrimary { start },
            2 => Phase::ToSecondary { start },
            _ => unreachable!(),
        }
    }
}

impl Strategy for SwapSABS {
    fn last_step(&self) -> Step {
        // A step for each AS, BA and SB step, where Scratch is fully filled.
        let blocks = self.num_pages.get() / self.scratch_pages.get();
        Step(blocks * 3)
    }

    fn plan(&self, step: Step) -> impl Iterator<Item = CopyOperation> {
        let phase = Phase::from_step(step, self.scratch_pages);

        let (from, to) = match phase {
            Phase::ToScratch { start } => (
                MemoryLocation {
                    slot: self.slot_primary,
                    page: start,
                },
                MemoryLocation {
                    slot: self.slot_scratch,
                    page: Page(0),
                },
            ),
            Phase::ToPrimary { start } => (
                MemoryLocation {
                    slot: self.slot_secondary,
                    page: start,
                },
                MemoryLocation {
                    slot: self.slot_primary,
                    page: start,
                },
            ),
            Phase::ToSecondary { start } => (
                MemoryLocation {
                    slot: self.slot_scratch,
                    page: Page(0),
                },
                MemoryLocation {
                    slot: self.slot_secondary,
                    page: start,
                },
            ),
        };

        (0..self.scratch_pages.get())
            .into_iter()
            .map(move |page| CopyOperation {
                from: MemoryLocation {
                    slot: from.slot,
                    page: Page(from.page.0 + page),
                },
                to: MemoryLocation {
                    slot: to.slot,
                    page: Page(to.page.0 + page),
                },
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Device, DeviceWithScratch,
        mock::{IMAGE_A, IMAGE_B, MockDevice},
    };

    use super::*;

    const PRIMARY: Slot = Slot(0);
    const SECONDARY: Slot = Slot(1);
    const SCRATCH: Slot = Slot(2);

    #[test]
    fn single_scratch() {
        let mut device = MockDevice::new();

        let strategy = SwapSABS {
            num_pages: device.page_count(),
            scratch_pages: device.scratch_page_count(),
            slot_primary: PRIMARY,
            slot_secondary: SECONDARY,
            slot_scratch: SCRATCH,
        };

        assert_eq!(device.primary, IMAGE_A);
        assert_eq!(device.secondary, IMAGE_B);

        for step_i in 0..strategy.last_step().0 {
            let step = Step(step_i);
            for operation in strategy.plan(step) {
                embassy_futures::block_on(async {
                    device.copy(operation).await.unwrap();
                })
            }
        }

        assert_eq!(device.primary, IMAGE_B);
        assert_eq!(device.secondary, IMAGE_A);

        assert!(device.wear.check_slot(PRIMARY, 1));
        assert!(device.wear.check_slot(SECONDARY, 1));
        assert!(
            device
                .wear
                .check_slot(SCRATCH, device.page_count().get() as usize)
        );
    }
}
