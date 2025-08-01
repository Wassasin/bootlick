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

use crate::{CopyOperation, MemoryLocation, Page, Slot, Step, strategies::Strategy};

pub struct SwapScootch {
    num_pages: u16,
    slot_primary: Slot,
    slot_secondary: Slot,
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
    pub const fn from_step(mut step: Step, num_pages: u16) -> Phase {
        if step.0 < num_pages {
            return Phase::Scootch(Page(step.0));
        }

        step.0 -= num_pages;

        // Copy the other pages in reverse order.
        let page = Page(num_pages - (step.0 / 2) - 1);
        if step.0 % 2 == 0 {
            Phase::ToPrimary(page)
        } else {
            Phase::ToSecondary(page)
        }
    }
}

impl SwapScootch {
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
        Step(self.num_pages * 3)
    }

    fn plan(&self, step: Step) -> CopyOperation {
        let phase = Phase::from_step(step, self.num_pages);

        // Convert a logical phase into a raw copy operation.
        match phase {
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
                    slot: self.slot_secondary,
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
                    slot: self.slot_secondary,
                    page,
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::Device;

    use super::*;

    const PAGE_COUNT: u16 = 3;

    const PRIMARY: Slot = Slot(0);
    const SECONDARY: Slot = Slot(1);
    const SCRATCH: Slot = Slot(2);

    const STRATEGY: SwapScootch = SwapScootch {
        num_pages: PAGE_COUNT,
        slot_primary: PRIMARY,
        slot_secondary: SECONDARY,
        slot_scratch: SCRATCH,
    };

    struct MockDevice {
        primary: [u8; PAGE_COUNT as usize],
        secondary: [u8; PAGE_COUNT as usize],
        scratch: [u8; 1],
    }

    const IMAGE_A: [u8; PAGE_COUNT as usize] = [0x01, 0x02, 0x03];
    const IMAGE_B: [u8; PAGE_COUNT as usize] = [0x04, 0x05, 0x06];

    impl MockDevice {
        pub const fn new() -> MockDevice {
            MockDevice {
                primary: IMAGE_A,
                secondary: IMAGE_B,
                scratch: [0xff],
            }
        }

        fn get_mut(&mut self, addr: MemoryLocation) -> &mut u8 {
            match addr.slot {
                Slot(0) => self.primary.as_mut_slice(),
                Slot(1) => self.secondary.as_mut_slice(),
                Slot(2) => self.scratch.as_mut_slice(),
                _ => unimplemented!(),
            }
            .get_mut(addr.page.0 as usize)
            .unwrap()
        }
    }

    impl Device for MockDevice {
        async fn copy(&mut self, operation: CopyOperation) -> Result<(), crate::Error> {
            let value = *self.get_mut(operation.from);
            *self.get_mut(operation.to) = value;
            Ok(())
        }

        async fn last_page(&self) -> Page {
            Page(PAGE_COUNT)
        }

        fn boot(_slot: Slot) -> ! {
            unimplemented!()
        }
    }

    #[test]
    fn scootch() {
        let mut device = MockDevice::new();

        assert_eq!(device.primary, IMAGE_A);
        assert_eq!(device.secondary, IMAGE_B);

        for step_i in 0..STRATEGY.last_step().0 {
            let step = Step(step_i);
            let operation = STRATEGY.plan(step);

            embassy_futures::block_on(async {
                device.copy(operation).await.unwrap();
            })
        }

        assert_eq!(device.primary, IMAGE_B);
        assert_eq!(device.secondary, IMAGE_A);
    }
}
