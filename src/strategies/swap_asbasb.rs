//! Strategy to swap two slots using 'AS BA SB', leaving both intact after finalizing.
//!
//! In other bootloaders also called 'swap scratch'.
//! It employs a 'scratch' partition as a temporary buffer for one of the pages.
//! This is an especially useful strategy if a memory type is available that is very wear resistant, like FRAM.
//!
//! The primary slot page is first copied over to the scratch memory, before writing the secondary slot page to the primary slot page.
//! Finally the scratch memory page is written to the secondary memory page.
//!
//! This results in the primary and secondary slots enduring a single erasure on every page for this strategy, whilst the scratch page endures `N` erasures, where `N` is the number of pages.
//!
//! **TODO** What if the scratch memory is several pages big?
