use crate::{CopyOperation, Device, MemoryLocation, Page, Slot};

pub const PAGE_COUNT: u16 = 3;

pub struct MockDevice {
    pub primary: [u8; PAGE_COUNT as usize],
    pub secondary: [u8; PAGE_COUNT as usize],
    pub scratch: [u8; 1],
}

pub const IMAGE_A: [u8; PAGE_COUNT as usize] = [0x01, 0x02, 0x03];
pub const IMAGE_B: [u8; PAGE_COUNT as usize] = [0x04, 0x05, 0x06];

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
