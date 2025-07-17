#![no_std]
#![no_main]

mod bsp;
mod fake;

use embassy_executor::Spawner;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    defmt::info!("Bootlicker");

    #[allow(unused)]
    let bsp::Peripherals { mut ext_flash, .. } = bsp::Peripherals::take(spawner);

    defmt::info!("device: {}", ext_flash.device_id().await.unwrap());

    let mut buf = [0u8; 256];
    ext_flash.read(1024, &mut buf).await.unwrap();

    defmt::info!("buf: {}", buf);

    ext_flash.erase_range(0, 4096).await.unwrap();
    for b in buf.iter_mut() {
        *b = 0x12;
    }
    ext_flash.write(1024, &buf).await.unwrap();

    ext_flash.read(1024, &mut buf).await.unwrap();
    defmt::info!("buf: {}", buf);

    loop {
        embassy_futures::yield_now().await;
    }
}
