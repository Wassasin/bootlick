use core::num::NonZeroU16;

use crate::{
    CopyOperation, Device, DeviceWithPrimarySlot, DeviceWithScratch, MemoryLocation, Slot,
    mock::WearTracker,
};

const PAGE_COUNT: NonZeroU16 = NonZeroU16::new(3).unwrap();
const SCRATCH_PAGE_COUNT: NonZeroU16 = NonZeroU16::new(1).unwrap();

pub struct MockDevice {
    pub primary: [u8; PAGE_COUNT.get() as usize],
    pub secondary: [u8; PAGE_COUNT.get() as usize],
    pub scratch: [u8; SCRATCH_PAGE_COUNT.get() as usize],
    pub wear: WearTracker,
}

pub const IMAGE_A: [u8; PAGE_COUNT.get() as usize] = [0x01, 0x02, 0x03];
pub const IMAGE_B: [u8; PAGE_COUNT.get() as usize] = [0x04, 0x05, 0x06];

pub const PRIMARY: Slot = Slot(0);
pub const SECONDARY: Slot = Slot(1);
pub const SCRATCH: Slot = Slot(2);

impl MockDevice {
    pub const fn new() -> MockDevice {
        MockDevice {
            primary: IMAGE_A,
            secondary: IMAGE_B,
            scratch: [0xff],
            wear: WearTracker::new(),
        }
    }

    fn get_mut(&mut self, addr: MemoryLocation) -> &mut u8 {
        match addr.slot {
            PRIMARY => self.primary.as_mut_slice(),
            SECONDARY => self.secondary.as_mut_slice(),
            SCRATCH => self.scratch.as_mut_slice(),
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

impl DeviceWithScratch for MockDevice {
    fn scratch_page_count(&self) -> NonZeroU16 {
        SCRATCH_PAGE_COUNT
    }

    fn get_scratch(&self) -> Slot {
        SCRATCH
    }
}

impl DeviceWithPrimarySlot for MockDevice {
    fn get_primary(&self) -> Slot {
        PRIMARY
    }
}
