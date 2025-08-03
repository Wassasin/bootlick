//! Strategy for selecting a slot using eXecute In Place.

use serde::{Deserialize, Serialize};

use crate::{Device, Slot, Step, strategies::Strategy};

/// Request to boot a target image.
///
/// When the target image fails to boot, will boot the backup.
///
/// * Note that if the backup is not provided, the device might brick itself.
/// * Note that the backup should have run successfully previously to ensure successful operation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub slot_target: Slot,
    pub slot_backup: Option<Slot>,
}

/// Strategy for selecting a slot using eXecute In Place.
///
/// This strategy does not copy any memory around, but directly jumps to the code in-memory.
/// A requirement however is that the device allows for executing code for the relevant memory.
/// Either the memory is already internal flash, or the device allows for executing code from an external device over for example QSPI.
///
/// Good to note is that with Xip execution the signature is not continuously verified.
/// Hence a man-in-the-middle might be possible if using external flash.
pub struct Xip {
    request: Request,
}

impl Xip {
    pub fn new(_device: &impl Device, request: Request) -> Self {
        Self { request }
    }
}

impl Strategy for Xip {
    fn last_step(&self) -> Step {
        Step(0)
    }

    fn plan(&self, _step: crate::Step) -> impl Iterator<Item = crate::CopyOperation> {
        core::iter::empty()
    }

    fn revert(self) -> Option<Self> {
        self.request.slot_backup.map(|slot_backup| Self {
            request: Request {
                slot_target: slot_backup,
                slot_backup: None,
            },
        })
    }
}
