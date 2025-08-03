use serde::{Deserialize, Serialize};

use crate::Step;

#[cfg(feature = "simple_state")]
pub mod simple;

#[derive(Serialize, Deserialize)]
pub struct Request<S> {
    /// The underlying strategy specific request.
    strategy: S,

    /// The step indicating the progress on the request execution.
    ///
    /// If set to the last step of a strategy, it indicates that a boot of the image will be attempted.
    /// If this attempt fails, it will set the revert bit and recover the previous image (if any).
    step: Step,

    /// Bit to indicate that the original request was attempted and failed.
    /// The steps now indicate how far along the strategy is in reverting to the previous (working) situation.
    revert: bool,
}

/// State as stored by the bootloader.
#[derive(Serialize, Deserialize)]
pub struct State<S> {
    /// Request indicating that the bootloader should perform a specific strategy.
    ///
    /// If no request is set, it will try to boot the primary image, if any exists.
    request: Option<Request<S>>,
}
