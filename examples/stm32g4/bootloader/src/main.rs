#![no_std]
#![no_main]

mod bsp;
mod fake;
mod partitions;
mod state;

use embassy_executor::Spawner;
use partition_manager::PartitionManager;

use crate::{
    partitions::{
        ExternalStorageConfig, ExternalStorageMap, InternalStorageConfig, InternalStorageMap,
    },
    state::{PersistentState, Slot, State},
};

use {defmt_rtt as _, panic_halt as _};

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

    let mut int_flash: PartitionManager<_> = PartitionManager::new(&mut int_flash);
    let mut ext_flash: PartitionManager<_> = PartitionManager::new(&mut ext_flash);

    let InternalStorageMap {
        mut bootloader,
        mut slot_primary,
    } = int_flash.map(InternalStorageConfig::new());
    let ExternalStorageMap {
        mut bl_state,
        mut bl_swap,
        mut slot_secundary,
    } = ext_flash.map(ExternalStorageConfig::new());

    let mut state = PersistentState::new(bl_state).await;

    defmt::info!("State: {}", state.get());

    state
        .store(State::Request {
            current: Slot(0),
            target: Slot(1),
        })
        .await;

    loop {
        match *state.get() {
            State::Initial => {
                boot_into(Slot(0)).await;
            }
            State::Trialing { target, old } => {
                // We never got here during update, but only after reboot, which means we failed.
                state
                    .store(State::Returning {
                        failed: target,
                        old,
                        step: 0,
                    })
                    .await
            }
            State::Failed { current, failed: _ } => boot_into(current).await,
            State::Confirmed { target } => boot_into(target).await,
            State::Request { current, target } => {
                state
                    .store(State::Swapping {
                        target,
                        old: current,
                        step: 0,
                    })
                    .await
            }
            State::Swapping { target, old, step } => {
                step_swapmove(old, target, step).await;

                if step < get_step_count() {
                    state
                        .store(State::Swapping {
                            target,
                            old,
                            step: step + 1,
                        })
                        .await;
                } else {
                    state.store(State::Trialing { target, old }).await;
                    boot_into(target).await;
                }
            }
            State::Returning { failed, old, step } => {
                step_swapmove(failed, old, step).await;

                if step < get_step_count() {
                    state
                        .store(State::Returning {
                            failed,
                            old,
                            step: step + 1,
                        })
                        .await;
                } else {
                    state
                        .store(State::Failed {
                            current: old,
                            failed,
                        })
                        .await;
                    boot_into(old).await;
                }
            }
        }
    }
}

async fn boot_into(slot: Slot) -> ! {
    defmt::info!("Boot into {}", slot);
    loop {
        embassy_futures::yield_now().await;
    }
}

async fn step_swapmove(a: Slot, b: Slot, step: u16) {
    defmt::info!("Step {}/{}", step, get_step_count());
}

fn get_step_count() -> u16 {
    // Move last sector to scratch
    // Scootch all pages one page
    //
    10
    // TODO
}
