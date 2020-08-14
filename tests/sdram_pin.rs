//! Tests SDRAM pin constraints apply correctly

mod dummy_pins;
use dummy_pins::*;

use stm32_fmc::*;

/// Dummy FmcPeripheral implementation for testing
struct DummyFMC;
unsafe impl FmcPeripheral for DummyFMC {
    const REGISTERS: *const () = 0 as *const ();
    fn enable(&mut self) {}
    fn source_clock_hz(&self) -> u32 {
        100_000_000
    }
}

macro_rules! fmc_pin_set {
    ($($p:ident),*) => {
        paste::item! {
            (
                $(
                    [< PinThats $p:upper>] {}
                ),*
            )
        }
    }
}

#[test]
/// SDRAM with 12 address pins, 4 banks
fn sdram_pins_12a_4b() {
    let fmc = DummyFMC {};
    let pins = fmc_pin_set!(
        // 12 address bits
        A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11,
        // 4 internal banks --------------------------------------
        BA0, BA1,
        // 32 bit data -------------------------------------------
        D0, D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D11, D12, D13, D14, D15,
        D16, D17, D18, D19, D20, D21, D22, D23, D24, D25, D26, D27, D28, D29,
        D30, D31,
        // NBL0-3 ------------------------------------------------
        NBL0, NBL1, NBL2, NBL3,
        // SDRAM Bank 0 ------------------------------------------
        SDCKE0, SDCLK, SDNCAS, SDNE0, SDNRAS, SDNWE
    );
    let chip = devices::is42s32800g_6::Is42s32800g {};

    // Check we can create a SDRAM
    Sdram::new(fmc, pins, chip);
}

#[test]
#[should_panic]
/// SDRAM with 12 address pins, 4 banks
fn sdram_pins_12a_4b_not_enough_adress_pins() {
    let fmc = DummyFMC {};
    let pins = fmc_pin_set!(
        // 11 bit address (!)
        A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10,
        // 4 internal banks --------------------------------------
        BA0, BA1,
        // 32 bit data -------------------------------------------
        D0, D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D11, D12, D13, D14, D15,
        D16, D17, D18, D19, D20, D21, D22, D23, D24, D25, D26, D27, D28, D29,
        D30, D31,
        // NBL0-3 ------------------------------------------------
        NBL0, NBL1, NBL2, NBL3,
        // SDRAM Bank 0 ------------------------------------------
        SDCKE0, SDCLK, SDNCAS, SDNE0, SDNRAS, SDNWE
    );
    let chip = devices::is42s32800g_6::Is42s32800g {};

    // Check we can create a SDRAM
    Sdram::new(fmc, pins, chip);
}

#[test]
#[should_panic]
/// SDRAM with 12 address pins, 4 banks
fn sdram_pins_12a_4b_not_enough_bank_pins() {
    let fmc = DummyFMC {};
    let pins = fmc_pin_set!(
        // 12 address bits
        A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11,
        // 2 internal banks (!) -----------------------------------
        BA0,
        // 32 bit data --------------------------------------------
        D0, D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D11, D12, D13, D14, D15,
        D16, D17, D18, D19, D20, D21, D22, D23, D24, D25, D26, D27, D28, D29,
        D30, D31,
        // NBL0-3 -------------------------------------------------
        NBL0, NBL1, NBL2, NBL3,
        // SDRAM Bank 0 -------------------------------------------
        SDCKE0, SDCLK, SDNCAS, SDNE0, SDNRAS, SDNWE
    );
    let chip = devices::is42s32800g_6::Is42s32800g {};

    // Check we can create a SDRAM
    Sdram::new(fmc, pins, chip);
}
