pub mod multi_scratch;
pub mod single_scratch;
pub mod tri_slot;

use std::collections::BTreeMap;

use crate::{MemoryLocation, Slot};

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
