# Bootlick
Bootloader toolkit to create your own fit-for-purpose bootloader and associated tools.

*Bootlick is still in active development and not ready for use*

## Version stability
Bootlick is not stable across major versions. In general, once you instantiate and flash your bootloader, never try to update it.

## Design
The default workflow involves:
* creating your own library crate in which you instantiate Bootlick types.
* creating an application for your bootloader that depends on your library.
* using the library in your firmware application to support OTA or DFU operations.
* creating tooling that allow you to format and optionally sign application images.

### Properties
* Fault-tolerant design capable of handling power failures during image swapping.
* Pick-and-choose parts for your specific situation, or add your own say you want to use device specific peripherals.
* Bootlick is memory agnostic: use any type of memory you want. Your non-volatile NOR flash does not even need to be mapped to the normal address space!

### Non-goals
Bootlick does not aim to be the most space efficient design possible. Rather ease of use and clear semantics are more important.
