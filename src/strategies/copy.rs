//! Strategy to copy a slot from secondary to the primary slot, completely forgetting what is in the primary slot.
//!
//! This is useful if you have sufficient memory on the board to keep all versions around, but only one type of memory from which
//! the application can be executed. (for example: MCU flash and a on-pcb NOR flash module, for which the module has plenty of space left)
//!
//! Another advantage is that it does not require a scratch page.

use core::num::NonZeroU16;

use crate::{
    CopyOperation, Device, DeviceWithPrimarySlot, MemoryLocation, Page, Slot, Step,
    strategies::Strategy,
};

#[derive(Clone, Debug)]
pub struct Request {
    pub slot_secondary: Slot,
}

pub struct Copy {
    request: Request,
    num_pages: NonZeroU16,
    slot_primary: Slot,
}

impl Copy {
    pub fn new(device: &(impl Device + DeviceWithPrimarySlot), request: Request) -> Self {
        Self {
            request,
            num_pages: device.page_count(),
            slot_primary: device.get_primary(),
        }
    }
}

impl Strategy for Copy {
    fn last_step(&self) -> Step {
        // We only need two steps: one to copy all over, one to boot.
        // More steps are not necessary because on resume we can just start over.
        Step(1)
    }

    fn plan(&self, _step: Step) -> impl Iterator<Item = CopyOperation> {
        (0..self.num_pages.get())
            .into_iter()
            .map(Page)
            .map(move |page| CopyOperation {
                from: MemoryLocation {
                    slot: self.request.slot_secondary,
                    page,
                },
                to: MemoryLocation {
                    slot: self.slot_primary,
                    page,
                },
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perform_copy(device: &mut (impl Device + DeviceWithPrimarySlot), slot_secondary: Slot) {
        let strategy = Copy::new(device, Request { slot_secondary });

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
    fn test() {
        use crate::mock::single_scratch::{IMAGE_A, IMAGE_B, MockDevice, PRIMARY, SECONDARY};

        let mut device = MockDevice::new();

        assert_eq!(device.primary, IMAGE_A);
        assert_eq!(device.secondary, IMAGE_B);

        perform_copy(&mut device, SECONDARY);

        assert_eq!(device.primary, IMAGE_B);
        assert_eq!(device.secondary, IMAGE_B);

        assert!(device.wear.check_slot(PRIMARY, 1));
        assert!(device.wear.check_slot(SECONDARY, 0));
    }
}
