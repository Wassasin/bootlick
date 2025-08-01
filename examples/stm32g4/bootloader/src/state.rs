use defmt::Format;
use embedded_storage_async::nor_flash::NorFlash;
use sequential_storage::{
    cache::KeyPointerCache,
    map::{SerializationError, Value},
};
use serde::{Deserialize, Serialize};

// TODO optimize storage size

#[derive(Serialize, Deserialize, Format, Debug, PartialEq, Clone, Copy)]
pub struct Slot(pub u8);

#[derive(Serialize, Deserialize, Format, Debug, PartialEq)]
pub enum State {
    /// Fresh from the factory, and it will boot the first image.
    Initial,
    /// Swapped to new image and awaiting to confirm it.
    ///
    /// If the device reboots when in Trialing, it will go back to the old slot with state Failed.
    Trialing { target: Slot, old: Slot },
    /// An image failed to trial, and we are back to the previous image (which was confirmed).
    Failed { current: Slot, failed: Slot },
    /// An image was trialed and confirmed when running the image.
    Confirmed { target: Slot },
    /// An user requested that a `target` slot is booted.
    Request { current: Slot, target: Slot },
    /// We are in the process of swapping to a new image between `target` and `old`.
    ///
    /// The page number referenced here is the page size of the medium with the largest pages.
    Swapping { target: Slot, old: Slot, step: u16 },
    /// We are in the process of swapping to an old image from `failed` to `old`.
    ///
    /// The page number referenced here is the page size of the medium with the largest pages.
    Returning { failed: Slot, old: Slot, step: u16 },
}

impl State {
    pub const fn max_serialized_size() -> usize {
        // TODO
        64
    }
}

impl<'a> Value<'a> for State {
    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        let buffer = postcard::to_slice(self, buffer).map_err(|e| match e {
            postcard::Error::SerializeBufferFull => SerializationError::BufferTooSmall,
            // Unmapped error.
            _ => SerializationError::Custom(0),
        })?;

        Ok(buffer.len())
    }

    fn deserialize_from(buffer: &'a [u8]) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        postcard::from_bytes(buffer).map_err(|e| match e {
            // Provided buffer is too small.
            postcard::Error::DeserializeUnexpectedEnd => SerializationError::BufferTooSmall,
            // Data type mismatch between Value and what is stored on disk.
            postcard::Error::DeserializeBadVarint
            | postcard::Error::DeserializeBadBool
            | postcard::Error::DeserializeBadChar
            | postcard::Error::DeserializeBadUtf8
            | postcard::Error::DeserializeBadOption
            | postcard::Error::DeserializeBadEnum
            | postcard::Error::DeserializeBadEncoding => SerializationError::InvalidFormat,
            // Unmapped error.
            _ => SerializationError::Custom(0),
        })
    }
}

pub struct PersistentState<NVM: NorFlash> {
    state: State,
    nvm: NVM,
    nvm_cache: KeyPointerCache<2, (), 1>,
}

impl<NVM: NorFlash> PersistentState<NVM> {
    pub async fn new(mut nvm: NVM) -> Self {
        let nvm_size = nvm.capacity() as u32;
        let mut nvm_cache: KeyPointerCache<2, (), 1> = KeyPointerCache::new();

        let mut data_buffer = [0u8; State::max_serialized_size()];

        let state = sequential_storage::map::fetch_item::<(), State, _>(
            &mut nvm,
            0..nvm_size,
            &mut nvm_cache,
            &mut data_buffer,
            &(),
        )
        .await
        .unwrap();

        let state = match state {
            Some(state) => state,
            None => {
                defmt::debug!("State NVM does not contain value");
                State::Initial
            }
        };

        Self {
            state,
            nvm,
            nvm_cache,
        }
    }

    pub fn get(&self) -> &State {
        &self.state
    }

    pub async fn store(&mut self, state: State) {
        let mut data_buffer = [0u8; State::max_serialized_size()];
        let nvm_size = self.nvm.capacity() as u32;

        defmt::debug!("Storing {}", state);

        sequential_storage::map::store_item::<(), State, _>(
            &mut self.nvm,
            0..nvm_size,
            &mut self.nvm_cache,
            &mut data_buffer,
            &(),
            &state,
        )
        .await
        .unwrap();
        self.state = state;
    }
}
