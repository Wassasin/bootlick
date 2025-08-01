use std::collections::BTreeMap;

use crate::{CopyOperation, Device, MemoryLocation, Page, Slot};

pub const PAGE_COUNT: u16 = 3;

#[derive(Debug)]
pub struct WearTracker(BTreeMap<MemoryLocation, usize>);

impl WearTracker {
    pub const fn new() -> Self {
        WearTracker(BTreeMap::new())
    }

    pub fn increase(&mut self, addr: MemoryLocation) {
        if let Some(wear) = self.0.get_mut(&addr) {
            *wear += 1;
        } else {
            self.0.insert(addr, 1);
        }
    }

    /// Check wear on all pages of slot for worst wear.
    pub fn check_slot(&self, slot: Slot, wear_level: usize) -> bool {
        self.0
            .iter()
            .filter(|(addr, _)| addr.slot == slot)
            .all(|(_, v)| *v <= wear_level)
    }
}

pub struct MockDevice {
    pub primary: [u8; PAGE_COUNT as usize],
    pub secondary: [u8; PAGE_COUNT as usize],
    pub scratch: [u8; 1],
    pub wear: WearTracker,
}

pub const IMAGE_A: [u8; PAGE_COUNT as usize] = [0x01, 0x02, 0x03];
pub const IMAGE_B: [u8; PAGE_COUNT as usize] = [0x04, 0x05, 0x06];

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

        self.wear.increase(operation.to);

        Ok(())
    }

    async fn last_page(&self) -> Page {
        Page(PAGE_COUNT)
    }

    fn boot(_slot: Slot) -> ! {
        unimplemented!()
    }
}
