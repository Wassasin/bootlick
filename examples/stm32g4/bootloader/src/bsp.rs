use core::mem;

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_stm32::{
    flash::{Blocking, Flash},
    gpio::{Level, Output, Speed},
    mode::Async,
    peripherals::IWDG,
    rcc::mux::Adcsel,
    spi::{self, Spi},
    time::Hertz,
    wdg::IndependentWatchdog,
    Config,
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use static_cell::StaticCell;
use w25::{Q, W25};

use crate::fake::Nothing;

pub struct Peripherals {
    pub watchdog: IndependentWatchdog<'static, IWDG>,
    pub ext_flash: W25<
        Q,
        SpiDevice<'static, NoopRawMutex, Spi<'static, Async>, Output<'static>>,
        Nothing,
        Nothing,
    >,
    pub int_flash: Flash<'static, Blocking>,
}

impl Peripherals {
    pub fn take(_spawner: Spawner) -> Self {
        let mut config = Config::default();
        {
            use embassy_stm32::rcc::*;
            config.rcc.hsi = true;
            config.rcc.pll = Some(Pll {
                source: PllSource::HSI,
                prediv: PllPreDiv::DIV4,
                mul: PllMul::MUL40,
                divp: None,
                divq: None,
                divr: Some(PllRDiv::DIV2),
            });
            config.rcc.sys = Sysclk::PLL1_R;
            config.rcc.mux.adc12sel = Adcsel::SYS;
        }

        let p = embassy_stm32::init(config);

        let watchdog = IndependentWatchdog::new(p.IWDG, 1_000_000);

        let spi_bus = {
            let mut config = spi::Config::default();
            config.frequency = Hertz(20_000_000);
            Spi::new(p.SPI1, p.PB3, p.PB5, p.PB4, p.DMA1_CH1, p.DMA1_CH2, config)
        };
        static STATIC_BUS: StaticCell<Mutex<NoopRawMutex, Spi<'static, Async>>> = StaticCell::new();
        let spi_bus = STATIC_BUS.init(Mutex::new(spi_bus));

        let ext_flash = {
            let ncs = Output::new(p.PB6, Level::High, Speed::VeryHigh);
            let spi_device = SpiDevice::new(spi_bus, ncs);

            // Pull nWP high and forget the pin to leave it in that state.
            let nwp = Output::new(p.PA0, Level::High, Speed::Low);
            mem::forget(nwp);

            W25::<Q, _, _, _>::new(spi_device, Nothing, Nothing, 8 * 1024 * 1024).unwrap()
        };

        let int_flash = Flash::new_blocking(p.FLASH);

        Peripherals {
            watchdog,
            ext_flash,
            int_flash,
        }
    }
}
