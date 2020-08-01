//! HAL for Flexible memory controller (FMC)

/// FMC Bank Base Addresses. See RM0433 Figure 95.
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(unused)]
pub enum FmcBank {
    Bank1,
    Bank2,
    Bank3,
    Bank4,
    Bank5,
    Bank6,
}
impl FmcBank {
    /// Return a pointer to this FMC bank
    pub fn ptr(self) -> *mut u32 {
        use FmcBank::*;
        (match self {
            Bank1 => 0x6000_0000u32,
            Bank2 => 0x7000_0000u32,
            Bank3 => 0x8000_0000u32,
            Bank4 => 0x9000_0000u32, // Not used
            Bank5 => 0xC000_0000u32,
            Bank6 => 0xD000_0000u32,
        }) as *mut u32
    }
}

/// Set of pins for an SDRAM
pub trait PinsSdram {
    const EXTERNAL_BANK: u8;
    const NUMBER_INTERNAL_BANKS: u8;
    const ADDRESS_LINES: u8;
}

/// Set of pins for SDRAM on Bank 1 of FMC controller
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PinsSdramBank1<T>(pub T);
/// Set of pins for SDRAM on Bank 2 of FMC controller
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PinsSdramBank2<T>(pub T);

macro_rules! impl_16bit_sdram {
    ($($pins:tt: [$eBankN:expr, $ckeN:tt, $neN:tt,
        $nInternalB:expr $(, $pba1:ident, $ba1:tt)*]),+) => {
        $(
            #[rustfmt::skip]
            /// 16-bit SDRAM
            impl<PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10,
            PA11, PBA0, $($pba1,)* PD0, PD1, PD2, PD3, PD4, PD5, PD6, PD7, PD8,
            PD9, PD10, PD11, PD12, PD13, PD14, PD15, PNBL0, PNBL1, PSDCKEn,
            PSDCLK, PSDNCAS, PSDNEn, PSDNRAS, PSDNWE>
                PinsSdram
                for $pins<(PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10,
                     PA11, PBA0, $($pba1,)* PD0, PD1, PD2, PD3, PD4, PD5, PD6, PD7,
                     PD8, PD9, PD10, PD11, PD12, PD13, PD14, PD15, PNBL0, PNBL1,
                     PSDCKEn, PSDCLK, PSDNCAS, PSDNEn, PSDNRAS, PSDNWE)>
            where PA0: A0, PA1: A1, PA2: A2, PA3: A3, PA4: A4, PA5: A5, PA6: A6,
                  PA7: A7, PA8: A8, PA9: A9, PA10: A10, PA11: A11,
                  PBA0: BA0, $($pba1:$ba1,)*
                  PD0: D0, PD1: D1, PD2: D2, PD3: D3, PD4: D4, PD5: D5, PD6: D6,
                  PD7: D7, PD8: D8, PD9: D9, PD10: D10, PD11: D11, PD12: D12,
                  PD13: D13, PD14: D14, PD15: D15,
                  PNBL0: NBL0, PNBL1: NBL1, PSDCKEn: $ckeN, PSDCLK: SDCLK,
                  PSDNCAS: SDNCAS, PSDNEn: $neN, PSDNRAS: SDNRAS, PSDNWE: SDNWE {
                const ADDRESS_LINES: u8 = 12;
                const EXTERNAL_BANK: u8 = $eBankN;
                const NUMBER_INTERNAL_BANKS: u8 = $nInternalB;
            }
        )+
    }
}

macro_rules! impl_32bit_sdram {
    ($($pins:tt: [$eBankN:expr, $ckeN:tt, $neN:tt,
        $nInternalB:expr $(, $pba1:ident, $ba1:tt)*]),+) => {
        $(
            #[rustfmt::skip]
            /// 32-bit SDRAM
            impl<PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10,
            PA11, PBA0, $($pba1,)* PD0, PD1, PD2, PD3, PD4, PD5, PD6, PD7, PD8,
            PD9, PD10, PD11, PD12, PD13, PD14, PD15, PD16, PD17, PD18, PD19,
            PD20, PD21, PD22, PD23, PD24, PD25, PD26, PD27, PD28, PD29, PD30,
            PD31, PNBL0, PNBL1, PNBL2, PNBL3, PSDCKEn, PSDCLK, PSDNCAS,
            PSDNEn, PSDNRAS, PSDNWE>
                PinsSdram
                for $pins<(PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10,
                     PA11, PBA0, $($pba1,)* PD0, PD1, PD2, PD3, PD4, PD5, PD6, PD7,
                     PD8, PD9, PD10, PD11, PD12, PD13, PD14, PD15, PD16, PD17,
                     PD18, PD19, PD20, PD21, PD22, PD23, PD24, PD25, PD26, PD27,
                     PD28, PD29, PD30, PD31, PNBL0, PNBL1, PNBL2, PNBL3, PSDCKEn,
                     PSDCLK, PSDNCAS, PSDNEn, PSDNRAS, PSDNWE)>
            where PA0: A0, PA1: A1, PA2: A2, PA3: A3, PA4: A4, PA5: A5, PA6: A6,
                  PA7: A7, PA8: A8, PA9: A9, PA10: A10, PA11: A11,
                  PBA0: BA0, $($pba1:$ba1,)*
                  PD0: D0, PD1: D1, PD2: D2, PD3: D3, PD4: D4, PD5: D5, PD6: D6,
                  PD7: D7, PD8: D8, PD9: D9, PD10: D10, PD11: D11, PD12: D12,
                  PD13: D13, PD14: D14, PD15: D15, PD16: D16, PD17: D17,
                  PD18: D18, PD19: D19, PD20: D20, PD21: D21, PD22: D22,
                  PD23: D23, PD24: D24, PD25: D25, PD26: D26, PD27: D27,
                  PD28: D28, PD29: D29, PD30: D30, PD31: D31,
                  PNBL0: NBL0, PNBL1: NBL1, PNBL2: NBL2, PNBL3: NBL3,
                  PSDCKEn: $ckeN, PSDCLK: SDCLK,
                  PSDNCAS: SDNCAS, PSDNEn: $neN, PSDNRAS: SDNRAS, PSDNWE: SDNWE {
                const ADDRESS_LINES: u8 = 12;
                const EXTERNAL_BANK: u8 = $eBankN;
                const NUMBER_INTERNAL_BANKS: u8 = $nInternalB;
            }
        )+
    }
}

impl_16bit_sdram! {
    // 16-bit SDRAM with 12 address lines, BA0 only
    PinsSdramBank1: [1, SDCKE0, SDNE0, 2],
    PinsSdramBank2: [2, SDCKE1, SDNE1, 2],
    // 16-bit SDRAM with 12 address lines, BA0 and BA1
    PinsSdramBank1: [1, SDCKE0, SDNE0, 4, PBA1, BA1],
    PinsSdramBank2: [2, SDCKE1, SDNE1, 4, PBA1, BA1]
}

impl_32bit_sdram! {
    // 32-bit SDRAM with 12 address lines, BA0 only
    PinsSdramBank1: [1, SDCKE0, SDNE0, 2],
    PinsSdramBank2: [2, SDCKE1, SDNE1, 2],
    // 32-bit SDRAM with 12 address lines, BA0 and BA1
    PinsSdramBank1: [1, SDCKE0, SDNE0, 4, PBA1, BA1],
    PinsSdramBank2: [2, SDCKE1, SDNE1, 4, PBA1, BA1]
}

pub trait A0 {}
pub trait A1 {}
pub trait A10 {}
pub trait A11 {}
pub trait A12 {}
pub trait A13 {}
pub trait A14 {}
pub trait A15 {}
pub trait A16 {}
pub trait A17 {}
pub trait A18 {}
pub trait A19 {}
pub trait A2 {}
pub trait A20 {}
pub trait A21 {}
pub trait A22 {}
pub trait A23 {}
pub trait A24 {}
pub trait A25 {}
pub trait A3 {}
pub trait A4 {}
pub trait A5 {}
pub trait A6 {}
pub trait A7 {}
pub trait A8 {}
pub trait A9 {}
pub trait BA0 {}
pub trait BA1 {}
pub trait CLK {}
pub trait D0 {}
pub trait D1 {}
pub trait D10 {}
pub trait D11 {}
pub trait D12 {}
pub trait D13 {}
pub trait D14 {}
pub trait D15 {}
pub trait D16 {}
pub trait D17 {}
pub trait D18 {}
pub trait D19 {}
pub trait D2 {}
pub trait D20 {}
pub trait D21 {}
pub trait D22 {}
pub trait D23 {}
pub trait D24 {}
pub trait D25 {}
pub trait D26 {}
pub trait D27 {}
pub trait D28 {}
pub trait D29 {}
pub trait D3 {}
pub trait D30 {}
pub trait D31 {}
pub trait D4 {}
pub trait D5 {}
pub trait D6 {}
pub trait D7 {}
pub trait D8 {}
pub trait D9 {}
pub trait DA0 {}
pub trait DA1 {}
pub trait DA10 {}
pub trait DA11 {}
pub trait DA12 {}
pub trait DA13 {}
pub trait DA14 {}
pub trait DA15 {}
pub trait DA2 {}
pub trait DA3 {}
pub trait DA4 {}
pub trait DA5 {}
pub trait DA6 {}
pub trait DA7 {}
pub trait DA8 {}
pub trait DA9 {}
pub trait INT {}
pub trait NBL0 {}
pub trait NBL1 {}
pub trait NBL2 {}
pub trait NBL3 {}
pub trait NCE {}
pub trait NE1 {}
pub trait NE2 {}
pub trait NE3 {}
pub trait NE4 {}
pub trait NL {}
pub trait NOE {}
pub trait NWAIT {}
pub trait NWE {}
pub trait SDCKE0 {}
pub trait SDCKE1 {}
pub trait SDCLK {}
pub trait SDNCAS {}
pub trait SDNE0 {}
pub trait SDNE1 {}
pub trait SDNRAS {}
pub trait SDNWE {}

use crate::ral::fmc;
use crate::FmcPeripheral;

#[derive(Copy, Clone)]
pub(crate) struct FmcRegisters(usize);

impl FmcRegisters {
    #[inline(always)]
    pub fn new<FMC: FmcPeripheral>() -> Self {
        Self(FMC::REGISTERS as usize)
    }

    #[inline(always)]
    pub fn global(&self) -> &'static fmc::RegisterBlock {
        unsafe { &*(self.0 as *const _) }
    }
}
