//! Strategy for selecting a slot using eXecute In Place.

use crate::{Step, strategies::Strategy};

/// Strategy for selecting a slot using eXecute In Place.
///
/// This strategy does not copy any memory around, but directly jumps to the code in-memory.
/// A requirement however is that the device allows for executing code for the relevant memory.
/// Either the memory is already internal flash, or the device allows for executing code from an external device over for example QSPI.
///
/// Good to note is that with Xip execution the signature is not continuously verified.
/// Hence a man-in-the-middle might be possible if using external flash.
pub struct Xip;

impl Strategy for Xip {
    fn last_step(&self) -> Step {
        Step(0)
    }

    fn plan(&self, _step: crate::Step) -> impl Iterator<Item = crate::CopyOperation> {
        core::iter::empty()
    }
}
