//! Slot activation strategies like moving, copying or executing in place.

use crate::{CopyOperation, Step};

pub mod copy_primary;
pub mod swap_asbasb;
pub mod swap_rotate;
pub mod swap_scootch;
pub mod xip;

/// A slot activation strategy.
pub trait Strategy {
    /// The step which denotes that the swap has been completed, and that boot should occur.
    ///
    /// **Warning**: for this specific step and any subsequent step planning a `CopyOperation` is undefined behaviour.
    fn last_step(&self) -> Step;

    /// Plan the operations to be executed for a given step.
    fn plan(&self, step: Step) -> impl Iterator<Item = CopyOperation>;
}
