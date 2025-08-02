//! Strategy to swap three slots using 'rotate', leaving the source image intact.
//!
//! In the case that the device has only one persistant memory from which code can be executed,
//! and an external memory that has space for two or more images.
//!
//! The primary image slot is first copied over to the tertiary image slot.
//! The secondary image slot is then copied to the primary image slot, keeping the secondary image intact.
//!
//! This results in the primary and tertiary slots enduring a single erasure on every page for this strategy.
