//! Simple straightforward implementation of keeping the state.
//!
//! This implementation focusses on correctness and ease, contrary to efficiency and code size.
//! Uses `sequential-storage` and `postcard` to store and serialize/deserialize the bootloader state.

use core::marker::PhantomData;

use embedded_storage_async::nor_flash::NorFlash;
use sequential_storage::{cache::KeyPointerCache, map::SerializationError};
use serde::{Serialize, de::DeserializeOwned};

use crate::state::{State, StateStorage};

pub struct SimpleStateStorage<NVM, S> {
    nvm: NVM,
    nvm_cache: KeyPointerCache<2, (), 1>,
    _phantom: PhantomData<S>,
}

impl<NVM, S> SimpleStateStorage<NVM, S> {
    pub fn new(nvm: NVM) -> Self {
        Self {
            nvm,
            nvm_cache: KeyPointerCache::new(),
            _phantom: PhantomData,
        }
    }
}

const MAX_SERIALIZED_SIZE: usize = 64;

impl<'a, S> sequential_storage::map::Value<'a> for State<S>
where
    S: Serialize + DeserializeOwned,
{
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

impl<NVM, S> StateStorage<S> for SimpleStateStorage<NVM, S>
where
    NVM: NorFlash,
    S: Serialize + DeserializeOwned,
{
    type Error = sequential_storage::Error<NVM::Error>;

    async fn store(&mut self, state: &State<S>) -> Result<(), Self::Error> {
        let mut data_buffer = [0u8; MAX_SERIALIZED_SIZE];
        let nvm_size = self.nvm.capacity() as u32;

        sequential_storage::map::store_item::<(), State<S>, _>(
            &mut self.nvm,
            0..nvm_size,
            &mut self.nvm_cache,
            &mut data_buffer,
            &(),
            state,
        )
        .await
    }

    async fn fetch(&mut self) -> Result<State<S>, Self::Error> {
        let mut data_buffer = [0u8; MAX_SERIALIZED_SIZE];

        let nvm_size = self.nvm.capacity() as u32;
        let state = sequential_storage::map::fetch_item::<(), State<S>, _>(
            &mut self.nvm,
            0..nvm_size,
            &mut self.nvm_cache,
            &mut data_buffer,
            &(),
        )
        .await?;

        let state = match state {
            Some(state) => state,
            None => {
                // defmt::debug!("State NVM does not contain value");
                State { request: None }
            }
        };

        Ok(state)
    }
}
