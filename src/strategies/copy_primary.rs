//! Strategy to copy a slot from secondary to the primary slot, completely forgetting what is in the primary slot.
//!
//! This is useful if you have sufficient memory on the board to keep all versions around, but only one type of memory from which
//! the application can be executed. (for example: MCU flash and a on-pcb NOR flash module, for which the module has plenty of space left)
