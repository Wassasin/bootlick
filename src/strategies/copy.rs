//! Strategy to copy a slot from secondary to the primary slot, completely forgetting what is in the primary slot.
//!
//! This is useful if you have sufficient memory on the board to keep all versions around, but only one type of memory from which
//! the application can be executed. (for example: MCU flash and a on-pcb NOR flash module, for which the module has plenty of space left)
//!
//! Another advantage is that it does not require a scratch page.

use core::num::NonZeroU16;
use serde::{Deserialize, Serialize};

use crate::{
    CopyOperation, Device, DeviceWithPrimarySlot, MemoryLocation, Page, Slot, Step,
    strategies::Strategy,
};

/// Request to boot a secondary image, with an optional backup if the secondary image is invalid.
///
/// * Note that if the backup is not provided, the device might brick itself.
/// * Note that the backup should have run successfully previously to ensure successful operation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    /// The image to copy to the primary slot.
    pub slot_secondary: Slot,
    /// The image to copy to the primary slot when the secondary image fails to boot.
    pub slot_backup: Option<Slot>,
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

    fn revert(self) -> Option<Self> {
        if let Some(slot_backup) = self.request.slot_backup {
            Some(Self {
                request: Request {
                    slot_secondary: slot_backup,
                    slot_backup: None,
                },
                num_pages: self.num_pages,
                slot_primary: self.slot_primary,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perform_copy(device: &mut (impl Device + DeviceWithPrimarySlot), strategy: &Copy) {
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
        use crate::mock::tri_slot::{ALPHA, BETA, IMAGE_A, IMAGE_B, MockDevice, PRIMARY};

        let mut device = MockDevice::new();
        let strategy = Copy::new(
            &device,
            Request {
                slot_secondary: BETA,
                slot_backup: Some(ALPHA),
            },
        );

        assert_eq!(device.primary, IMAGE_A);
        assert_eq!(device.alpha, IMAGE_A);
        assert_eq!(device.beta, IMAGE_B);

        perform_copy(&mut device, &strategy);

        assert_eq!(device.primary, IMAGE_B);
        assert_eq!(device.alpha, IMAGE_A);
        assert_eq!(device.beta, IMAGE_B);

        assert!(device.wear.check_slot(PRIMARY, 1));
        assert!(device.wear.check_slot(ALPHA, 0));
        assert!(device.wear.check_slot(BETA, 0));

        let strategy = strategy.revert().unwrap();

        perform_copy(&mut device, &strategy);

        assert_eq!(device.primary, IMAGE_A);
        assert_eq!(device.alpha, IMAGE_A);
        assert_eq!(device.beta, IMAGE_B);
    }
}
