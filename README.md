# [Documentation](https://docs.rs/stm32-fmc)

# stm32-fmc

[![docs.rs](https://docs.rs/stm32-fmc/badge.svg)](https://docs.rs/stm32-fmc)
[![Crates.io](https://img.shields.io/crates/v/stm32-fmc.svg)](https://crates.io/crates/stm32-fmc)

Hardware Abstraction Layer for STM32 Memory Controllers (FMC/FSMC)

Currently only SDRAM functions are implemented.

**This crate is a work in progress! Contributions very welcome**

### SDRAM

The hardware supports up to 2 external SDRAM devices. This library
currently only supports 1, although it may be on either bank 1 or
2.

To pass pins to
[`Sdram::new`](https://docs.rs/stm32-fmc/latest/stm32-fmc/struct.Sdram.html#method.new),
create a tuple with the following ordering:

```rust
let pins = (
    // A0-A11
    pa0, ...
    // BA0-BA1
    // D0-D31
    // NBL0 - NBL3
    // SDCKE
    // SDCLK
    // SDNCAS
    // SDNE
    // SDRAS
    // SDNWE
);
```

External memories are defined by `SdramChip` implementations. There are already
several examples in the `devices/` folder.

### NOR Flash/PSRAM

TODO

### NAND Flash

TODO

## Implementing

See the [docs]((https://docs.rs/stm32-fmc)

<!-- ```rust -->
<!--     let mut sdram = -->
<!--         stm32_fmc::Sdram::new(fmc, fmc_io, is42s32800g_6::Is42s32800g {}); -->
<!-- ``` -->

<!-- Or use new_unchecked: -->

<!-- ```rust -->
<!--     let mut sdram = -->
<!--         stm32_fmc::Sdram::new_unchecked(fmc, is42s32800g_6::Is42s32800g {}); -->
<!-- ``` -->


<!-- ### IO Setup -->

<!-- IO is constructed by configuring each pin as high speed and -->
<!-- assigning to the FMC block. -->

<!-- ```rust -->
<!--     let pa0 = gpioa.pa0.into_push_pull_output() -->
<!--         .set_speed(Speed::VeryHigh) -->
<!--         .into_alternate_af12() -->
<!--         .internal_pull_up(true); -->
<!-- ``` -->

<!-- Then contruct a PinSdram type from the required pins. They must be -->
<!-- specified in the order given here. -->


<!-- See the [examples](examples) for an ergonomic method using macros. -->

# Usage

Follow the documention in your HAL to initialise the FMC.

Once you have an `Sdram` type from your HAL, you can:

* Initialise it, which returns a raw pointer
* Convert the raw pointer to a sized slice using `from_raw_parts_mut`

```rust
let ram = unsafe {
    // Initialise controller and SDRAM
    let ram_ptr: *mut u32 = sdram.init(&mut delay, ccdr.clocks);

    // 32 MByte = 256Mbit SDRAM = 8M u32 words
    slice::from_raw_parts_mut(ram_ptr, 8 * 1024 * 1024)
};
```


## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
