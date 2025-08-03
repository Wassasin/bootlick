use core::num::NonZeroU16;

use crate::{
    CopyOperation, Device, DeviceWithPrimarySlot, MemoryLocation, Slot, mock::WearTracker,
};

const PAGE_COUNT: NonZeroU16 = NonZeroU16::new(3).unwrap();

pub struct MockDevice {
    pub primary: [u8; PAGE_COUNT.get() as usize],
    pub alpha: [u8; PAGE_COUNT.get() as usize],
    pub beta: [u8; PAGE_COUNT.get() as usize],
    pub wear: WearTracker,
}

pub const IMAGE_A: [u8; PAGE_COUNT.get() as usize] = [0x01, 0x02, 0x03];
pub const IMAGE_B: [u8; PAGE_COUNT.get() as usize] = [0x04, 0x05, 0x06];

pub const PRIMARY: Slot = Slot(0);
pub const ALPHA: Slot = Slot(1);
pub const BETA: Slot = Slot(2);

impl MockDevice {
    pub const fn new() -> MockDevice {
        MockDevice {
            primary: IMAGE_A,
            alpha: IMAGE_A,
            beta: IMAGE_B,
            wear: WearTracker::new(),
        }
    }

    fn get_mut(&mut self, addr: MemoryLocation) -> &mut u8 {
        match addr.slot {
            PRIMARY => self.primary.as_mut_slice(),
            ALPHA => self.alpha.as_mut_slice(),
            BETA => self.beta.as_mut_slice(),
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

        self.wear.increase(operation.to);

        Ok(())
    }

    fn boot(self, _slot: Slot) -> ! {
        unimplemented!()
    }

    fn page_count(&self) -> core::num::NonZeroU16 {
        PAGE_COUNT
    }
}

impl DeviceWithPrimarySlot for MockDevice {
    fn get_primary(&self) -> Slot {
        PRIMARY
    }
}
