//! Hardware Abstraction Layer for Flexible Memory Controller (FMC) on the
//! STM32H7
//!
//! Currently only SDRAM functions are implemented.
//!
//! This crate depends on the GPIO, Clock and Delay functionality from
//! [`stm32h7xx-hal`].
//!
//! # SDRAM
//!
//! The H7 supports up to 2 external SDRAM devices. This library
//! currently only supports 1, although it may be on either bank 1 or
//! 2.
//!
//! ## IO Setup
//!
//! IO is constructed by configuring each pin as high speed and
//! assigning to the FMC block (usually AF12).
//!
//! ```rust
//!     let pa0 = gpioa.pa0.into_push_pull_output()
//!         .set_speed(Speed::VeryHigh)
//!         .into_alternate_af12()
//!         .internal_pull_up(true);
//! ```
//!
//! Then contruct a PinSdram type from the required pins. They must be
//! specified in the order given here.
//!
//! ```rust
//!     let fmc_io = stm32h7_fmc::PinsSdramBank1(
//!         (
//!             // A0-A11
//!             pa0, ...
//!             // BA0-BA1
//!             // D0-D31
//!             // NBL0 - NBL3
//!             // SDCKE
//!             // SDCLK
//!             // SDNCAS
//!             // SDNE
//!             // SDRAS
//!             // SDNWE
//!         )
//!     );
//! ```
//!
//! See the [examples](examples) for an ergonomic method using macros.
//!
//! ## Usage
//!
//! First create a new SDRAM from the FMC peripheral, IO and SDRAM
//! device constants.
//!
//! ```rust
//!     let mut sdram =
//!         stm32h7_fmc::Sdram::new(dp.FMC, fmc_io, is42s32800g_6::Is42s32800g {});
//! ```
//!
//! Then initialise the controller and the SDRAM device. Convert the
//! raw pointer to a sized slice using `from_raw_parts_mut`.
//!
//!
//! ```rust
//!     let ram = unsafe {
//!         // Initialise controller and SDRAM
//!         let ram_ptr: *mut u32 = sdram.init(&mut delay, ccdr.clocks);
//!
//!         // 32 MByte = 256Mbit SDRAM = 8M u32 words
//!         slice::from_raw_parts_mut(ram_ptr, 8 * 1024 * 1024)
//!     };
//! ```
//!
//!
//! ## License
//!
//! Licensed under either of
//!
//!  * Apache License, Version 2.0
//!    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
//!  * MIT license
//!    ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
//!
//! at your option.
//!
//! ## Contribution
//!
//! Unless you explicitly state otherwise, any contribution
//! intentionally submitted for inclusion in the work by you, as
//! defined in the Apache-2.0 license, shall be dual licensed as
//! above, without any additional terms or conditions.
//!
//! [`stm32h7xx-hal`]: https://crates.io/crates/stm32h7xx-hal
#![no_std]
// rustc lints.
#![warn(
    bare_trait_objects,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_qualifications,
    unused_results
)]

#[macro_use]
mod macros;

mod fmc;
pub use fmc::*;

mod sdram;
pub use sdram::{Sdram, SdramChip};

mod devices;
pub use devices::*;

mod ral;

/// A trait for device-specific FMC peripherals. Implement this to add support
/// for a new hardware platform. Peripherals that have this trait must have the
/// same register block as STM32 FMC peripherals.
pub unsafe trait FmcPeripheral: Send + Sync {
    /// Pointer to the register block
    const REGISTERS: *const ();

    /// Enables the FMC on its peripheral bus
    fn enable(&mut self);

    /// Enables the FMC memory controller (not always required)
    fn memory_controller_enable(&mut self) {}

    /// The frequency of the clock used as a source for the fmc_clk.
    ///
    /// F4/F7/G4: hclk
    /// H7: fmc_ker_ck
    fn source_clock_hz(&self) -> u32;
}
