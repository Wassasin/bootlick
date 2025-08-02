//! Strategy to swap two slots using 'scootch', leaving both intact after finalizing.
//!
//! In other bootloaders also called 'swap move'.
//! It employs a 'scratch' partition as a temporary buffer for one of the pages.
//! In order to limit the wear on this specific partition, the primary slot is first scootched over
//! by one page, before copying the secondary slot over.
//!
//! This results in the first slot enduring two erasures on every page for this strategy, and the second slot enduring a single erasure.
//!
//! **TODO** Hence it is beneficial to select the slot with the better wear resistance as the primary slot.
//!
//! **TODO** Does it help if the scratch memory spans multiple pages? => number of steps?

use core::num::NonZeroU16;

use crate::{
    CopyOperation, Device, DeviceWithPrimarySlot, DeviceWithScratch, MemoryLocation, Page, Slot,
    Step, strategies::Strategy,
};

#[derive(Clone, Debug)]
pub struct Request {
    pub slot_secondary: Slot,
}

pub struct SwapScootch {
    request: Request,
    num_pages: NonZeroU16,
    slot_primary: Slot,
    slot_scratch: Slot,
}

/// Logical phases for the strategy to execute, to decouple raw steps from behaviour in a logical manner.
#[derive(Debug)]
enum Phase {
    /// Scootch primary down one page, the first being scootched to the scratch.
    Scootch(Page),
    /// Copy from secondary to primary.
    ToPrimary(Page),
    /// Copy to secondary from either primary or scratch, considering that the primary has been scootched.
    ToSecondary(Page),
}

impl Phase {
    pub const fn from_step(mut step: Step, num_pages: NonZeroU16) -> Phase {
        if step.0 < num_pages.get() {
            return Phase::Scootch(Page(step.0));
        }

        step.0 -= num_pages.get();

        // Copy the other pages in reverse order.
        let page = Page(num_pages.get() - (step.0 / 2) - 1);
        if step.0 % 2 == 0 {
            Phase::ToPrimary(page)
        } else {
            Phase::ToSecondary(page)
        }
    }
}

impl SwapScootch {
    pub fn new(
        device: &(impl Device + DeviceWithScratch + DeviceWithPrimarySlot),
        request: Request,
    ) -> Self {
        Self {
            num_pages: device.page_count(),
            request,
            slot_primary: device.get_primary(),
            slot_scratch: device.get_scratch(),
        }
    }

    const fn scratch_location(&self) -> MemoryLocation {
        // TODO what if scratch is more than one page large?
        MemoryLocation {
            slot: self.slot_scratch,
            page: Page(0),
        }
    }
}

impl Strategy for SwapScootch {
    fn last_step(&self) -> Step {
        // A single move for scootch, and two copies for swap, plus a single step for boot.
        Step(self.num_pages.get() * 3)
    }

    fn plan(&self, step: Step) -> impl Iterator<Item = CopyOperation> {
        let phase = Phase::from_step(step, self.num_pages);

        // Convert a logical phase into a raw copy operation.
        let op = match phase {
            Phase::Scootch(page) => CopyOperation {
                from: MemoryLocation {
                    slot: self.slot_primary,
                    page,
                },
                to: if page == Page(0) {
                    self.scratch_location()
                } else {
                    MemoryLocation {
                        slot: self.slot_primary,
                        page: Page(page.0 - 1),
                    }
                },
            },
            // To primary slot is copied 1:1, meaning the same page is copied from secondary.
            Phase::ToPrimary(page) => CopyOperation {
                from: MemoryLocation {
                    slot: self.request.slot_secondary,
                    page,
                },
                to: MemoryLocation {
                    slot: self.slot_primary,
                    page,
                },
            },
            // To secondary the page is located in the N-1 page on the primary, of which page 0 is located in scratch.
            Phase::ToSecondary(page) => CopyOperation {
                from: if page == Page(0) {
                    self.scratch_location()
                } else {
                    MemoryLocation {
                        slot: self.slot_primary,
                        page: Page(page.0 - 1),
                    }
                },
                to: MemoryLocation {
                    slot: self.request.slot_secondary,
                    page,
                },
            },
        };

        core::iter::once(op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_scratch() {
        use crate::mock::single_scratch::{
            IMAGE_A, IMAGE_B, MockDevice, PRIMARY, SCRATCH, SECONDARY,
        };

        let mut device = MockDevice::new();

        let strategy = SwapScootch::new(
            &device,
            Request {
                slot_secondary: SECONDARY,
            },
        );

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

        assert!(device.wear.check_slot(PRIMARY, 2));
        assert!(device.wear.check_slot(SECONDARY, 1));
        assert!(device.wear.check_slot(SCRATCH, 1));
    }
}
