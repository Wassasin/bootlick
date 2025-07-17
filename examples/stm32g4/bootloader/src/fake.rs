use defmt::Format;
use embedded_hal::digital::{ErrorType, OutputPin};

/// Our flash chip has no HOLD and WP pins mapped, make
/// a fake GPIO for the `W25q32jv` to hold on to.
#[derive(Debug, Format)]
pub struct Nothing;

impl ErrorType for Nothing {
    type Error = core::convert::Infallible;
}

impl OutputPin for Nothing {
    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
