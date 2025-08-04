#![no_std]
#![no_main]

mod bsp;
mod fake;
mod partitions;

use bootlick::{
    state::{simple::SimpleStateStorage, State, StateStorage},
    strategies::swap_scootch::{self, SwapScootch},
    Device, DeviceWithPrimarySlot, DeviceWithScratch, MemoryLocation, Slot,
};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_stm32::{flash::Blocking, gpio::Output, mode::Async, spi::Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use partition_manager::{Partition, PartitionManager, RW};
use w25::W25;

use crate::{
    fake::Nothing,
    partitions::{
        ExternalStorageConfig, ExternalStorageMap, InternalStorageConfig, InternalStorageMap,
    },
};

use {defmt_rtt as _, panic_halt as _};

struct AsyncFlashAdapter<T>(T);

impl<T: embedded_storage::nor_flash::ErrorType> embedded_storage_async::nor_flash::ErrorType
    for AsyncFlashAdapter<T>
{
    type Error = T::Error;
}

impl<T: embedded_storage::nor_flash::ReadNorFlash> embedded_storage_async::nor_flash::ReadNorFlash
    for AsyncFlashAdapter<T>
{
    const READ_SIZE: usize = T::READ_SIZE;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.0.read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        self.0.capacity()
    }
}

impl<T: embedded_storage::nor_flash::NorFlash> embedded_storage_async::nor_flash::NorFlash
    for AsyncFlashAdapter<T>
{
    const WRITE_SIZE: usize = T::WRITE_SIZE;
    const ERASE_SIZE: usize = T::ERASE_SIZE;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.0.erase(from, to)
    }

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.0.write(offset, bytes)
    }
}

struct ThisDevice<'a> {
    slot_primary: Partition<
        'a,
        AsyncFlashAdapter<embassy_stm32::flash::Flash<'static, Blocking>>,
        RW,
        NoopRawMutex,
    >,
    slot_scratch: Partition<
        'a,
        W25<
            w25::Q,
            SpiDevice<'static, NoopRawMutex, Spi<'static, Async>, Output<'static>>,
            Nothing,
            Nothing,
        >,
        RW,
        NoopRawMutex,
    >,
}

const SLOT_PRIMARY: Slot = Slot(0);
const SLOT_SECONDARY: Slot = Slot(1);
const SLOT_SCRATCH: Slot = Slot(2);

impl ThisDevice<'_> {
    async fn erase_page(&mut self, loc: MemoryLocation) {
        match loc.slot {}
    }
}

impl Device for ThisDevice<'_> {
    async fn copy(&mut self, operation: bootlick::CopyOperation) -> Result<(), bootlick::Error> {
        let bootlick::CopyOperation { from, to };
    }

    fn boot(self, slot: Slot) -> ! {
        defmt::info!("Boot into {}", slot);
        loop {
            cortex_m::asm::wfe();
        }
    }

    fn page_count(&self) -> core::num::NonZeroU16 {
        use embedded_storage_async::nor_flash::{NorFlash, ReadNorFlash};
        core::num::NonZeroU16::new(
            (self.slot_primary.capacity()
                / AsyncFlashAdapter::<embassy_stm32::flash::Flash<'static, Blocking>>::ERASE_SIZE)
                as u16,
        )
        .unwrap()
    }
}

impl DeviceWithPrimarySlot for ThisDevice<'_> {
    fn get_primary(&self) -> Slot {
        SLOT_PRIMARY
    }
}

impl DeviceWithScratch for ThisDevice<'_> {
    fn scratch_page_count(&self) -> core::num::NonZeroU16 {
        use embedded_storage_async::nor_flash::{NorFlash, ReadNorFlash};
        core::num::NonZeroU16::new(
            (self.slot_scratch.capacity()
                / AsyncFlashAdapter::<embassy_stm32::flash::Flash<'static, Blocking>>::ERASE_SIZE)
                as u16,
        )
        .unwrap()
    }

    fn get_scratch(&self) -> Slot {
        Slot(2)
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    defmt::info!("Bootlicker");

    #[allow(unused)]
    let bsp::Peripherals {
        mut int_flash,
        mut ext_flash,
        ..
    } = bsp::Peripherals::take(spawner);

    defmt::info!("device: {}", ext_flash.device_id().await.unwrap());

    let mut int_flash: PartitionManager<_> = PartitionManager::new(AsyncFlashAdapter(int_flash));
    let mut ext_flash: PartitionManager<_> = PartitionManager::new(ext_flash);

    let InternalStorageMap {
        mut bootloader,
        mut slot_primary,
    } = int_flash.map(InternalStorageConfig::new());
    let ExternalStorageMap {
        mut bl_state,
        mut bl_swap,
        mut slot_secundary,
    } = ext_flash.map(ExternalStorageConfig::new());

    let mut state_storage = SimpleStateStorage::new(bl_state);

    let state: State<swap_scootch::Request> = state_storage.fetch().await.unwrap();
    let device = ThisDevice {
        slot_primary,
        slot_scratch: bl_swap,
    };

    if let Some(request) = state.request {
        let strategy = SwapScootch::new(&device, request.strategy);

        todo!()
    } else {
        defmt::info!("No request active, boot to primary!");
        let primary = device.get_primary();
        device.boot(primary)
    }
}
