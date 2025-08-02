//! Strategy to swap three slots using 'rotate', leaving the source image intact.
//!
//! In the case that the device has only one persistant memory from which code can be executed,
//! and an external memory that has space for two or more images.
//!
//! The primary (A) image slot is first copied over to the tertiary (C) image slot.
//! The secondary (B) image slot is then copied to the primary (A) image slot, keeping the secondary (B) image intact.
//!
//! `C <- A <- B`
//!
//! This results in the primary and tertiary slots enduring a single erasure on every page for this strategy.
