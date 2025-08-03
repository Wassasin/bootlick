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

use core::num::NonZeroU16;

use serde::{Deserialize, Serialize};

use crate::{
    CopyOperation, Device, DeviceWithPrimarySlot, DeviceWithScratch, MemoryLocation, Page, Slot,
    Step, strategies::Strategy,
};

/// Request to boot a secondary image.
///
/// When the secondary image fails to boot, will perform the swap again, restoring the original situation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub slot_secondary: Slot,
}

pub struct SwapSABS {
    request: Request,
    num_pages: NonZeroU16,
    scratch_pages: NonZeroU16,
    slot_primary: Slot,
    slot_scratch: Slot,
}

/// Logical phases for the strategy to execute, to decouple raw steps from behaviour in a logical manner.
#[derive(Debug)]
enum Phase {
    A2S,
    B2A,
    S2B,
}

impl Phase {
    /// Get the current destination and starting page from the step number.
    pub const fn from_step(step: Step, scratch_pages: NonZeroU16) -> (Phase, Page) {
        let destination = match step.0 % 3 {
            0 => Phase::A2S,
            1 => Phase::B2A,
            2 => Phase::S2B,
            _ => unreachable!(),
        };

        let start = Page((step.0 / 3) * scratch_pages.get());

        (destination, start)
    }
}

impl SwapSABS {
    pub fn new(
        device: &(impl Device + DeviceWithScratch + DeviceWithPrimarySlot),
        request: Request,
    ) -> Self {
        Self {
            request,
            num_pages: device.page_count(),
            scratch_pages: device.scratch_page_count(),
            slot_primary: device.get_primary(),
            slot_scratch: device.get_scratch(),
        }
    }
}

impl Strategy for SwapSABS {
    fn last_step(&self) -> Step {
        // Note(div_ceil): we might need to partially use the scratch pages for the final segment,
        // if it is not a neat multiple.
        let blocks = self.num_pages.get().div_ceil(self.scratch_pages.get());

        // A step for each AS, BA and SB step, where Scratch is fully filled.
        Step(blocks * 3)
    }

    fn plan(&self, step: Step) -> impl Iterator<Item = CopyOperation> {
        let (phase, start) = Phase::from_step(step, self.scratch_pages);

        let (from, to) = match phase {
            Phase::A2S => (
                MemoryLocation {
                    slot: self.slot_primary,
                    page: start,
                },
                MemoryLocation {
                    slot: self.slot_scratch,
                    page: Page(0),
                },
            ),
            Phase::B2A => (
                MemoryLocation {
                    slot: self.request.slot_secondary,
                    page: start,
                },
                MemoryLocation {
                    slot: self.slot_primary,
                    page: start,
                },
            ),
            Phase::S2B => (
                MemoryLocation {
                    slot: self.slot_scratch,
                    page: Page(0),
                },
                MemoryLocation {
                    slot: self.request.slot_secondary,
                    page: start,
                },
            ),
        };

        // How many pages do we have left to move in order to finish?
        let pages_left = self.num_pages.get() - start.0;

        // How many pages are we doing in this step?
        let pages_now = u16::min(pages_left, self.scratch_pages.get());

        (0..pages_now).into_iter().map(move |page| CopyOperation {
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

    fn revert(self) -> Option<Self> {
        // Reversion of swapping is the same operation.
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Device, DeviceWithScratch};

    use super::*;

    fn perform_copy(
        device: &mut (impl Device + DeviceWithScratch + DeviceWithPrimarySlot),
        strategy: &SwapSABS,
    ) {
        for step_i in 0..strategy.last_step().0 {
            let step = Step(step_i);
            for operation in strategy.plan(step) {
                embassy_futures::block_on(async {
                    device.copy(operation).await.unwrap();
                })
            }
        }
    }

    #[test]
    fn single_scratch() {
        use crate::mock::single_scratch::{
            IMAGE_A, IMAGE_B, MockDevice, PRIMARY, SCRATCH, SECONDARY,
        };

        let mut device = MockDevice::new();
        let strategy = SwapSABS::new(
            &device,
            Request {
                slot_secondary: SECONDARY,
            },
        );

        assert_eq!(device.primary, IMAGE_A);
        assert_eq!(device.secondary, IMAGE_B);

        perform_copy(&mut device, &strategy);

        assert_eq!(device.primary, IMAGE_B);
        assert_eq!(device.secondary, IMAGE_A);

        assert!(device.wear.check_slot(PRIMARY, 1));
        assert!(device.wear.check_slot(SECONDARY, 1));
        assert!(
            device
                .wear
                .check_slot(SCRATCH, device.page_count().get() as usize)
        );

        let strategy = strategy.revert().unwrap();

        perform_copy(&mut device, &strategy);

        assert_eq!(device.primary, IMAGE_A);
        assert_eq!(device.secondary, IMAGE_B);
    }

    #[test]
    fn multi_scratch() {
        use crate::mock::multi_scratch::{
            IMAGE_A, IMAGE_B, MockDevice, PRIMARY, SCRATCH, SECONDARY,
        };

        let mut device = MockDevice::new();
        let strategy = SwapSABS::new(
            &device,
            Request {
                slot_secondary: SECONDARY,
            },
        );

        assert_eq!(device.primary, IMAGE_A);
        assert_eq!(device.secondary, IMAGE_B);

        perform_copy(&mut device, &strategy);

        assert_eq!(device.primary, IMAGE_B);
        assert_eq!(device.secondary, IMAGE_A);

        assert!(device.wear.check_slot(PRIMARY, 1));
        assert!(device.wear.check_slot(SECONDARY, 1));
        assert!(
            device.wear.check_slot(
                SCRATCH,
                device
                    .page_count()
                    .get()
                    .div_ceil(device.scratch_page_count().get()) as usize
            )
        );

        let strategy = strategy.revert().unwrap();

        perform_copy(&mut device, &strategy);

        assert_eq!(device.primary, IMAGE_A);
        assert_eq!(device.secondary, IMAGE_B);
    }
}
