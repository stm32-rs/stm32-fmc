//! HAL for Flexible memory controller (FMC)

use crate::stm32::FMC;

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

/// FMC controller
#[allow(missing_debug_implementations)]
pub struct Fmc {
    /// Flexible memory controller (FMC)
    pub(crate) fmc: FMC,
    // /// FMC clock selection
    // clk_sel: (), //rec::FmcClkSel,
}

/// Set of pins for an SDRAM
pub trait PinsSdram<FMC> {
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
            impl<FMC, PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10,
            PA11, PBA0, $($pba1,)* PD0, PD1, PD2, PD3, PD4, PD5, PD6, PD7, PD8,
            PD9, PD10, PD11, PD12, PD13, PD14, PD15, PNBL0, PNBL1, PSDCKEn,
            PSDCLK, PSDNCAS, PSDNEn, PSDNRAS, PSDNWE>
                PinsSdram<FMC>
                for $pins<(PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10,
                     PA11, PBA0, $($pba1,)* PD0, PD1, PD2, PD3, PD4, PD5, PD6, PD7,
                     PD8, PD9, PD10, PD11, PD12, PD13, PD14, PD15, PNBL0, PNBL1,
                     PSDCKEn, PSDCLK, PSDNCAS, PSDNEn, PSDNRAS, PSDNWE)>
            where PA0: A0<FMC>, PA1: A1<FMC>, PA2: A2<FMC>, PA3: A3<FMC>, PA4:
            A4<FMC>, PA5: A5<FMC>, PA6: A6<FMC>, PA7: A7<FMC>, PA8: A8<FMC>, PA9:
            A9<FMC>, PA10: A10<FMC>, PA11: A11<FMC>, PBA0: BA0<FMC>,
            $($pba1:$ba1<FMC>,)*
            PD0: D0<FMC>, PD1: D1<FMC>, PD2: D2<FMC>, PD3: D3<FMC>, PD4:
            D4<FMC>, PD5: D5<FMC>, PD6: D6<FMC>, PD7: D7<FMC>, PD8: D8<FMC>, PD9:
            D9<FMC>, PD10: D10<FMC>, PD11: D11<FMC>, PD12: D12<FMC>, PD13:
            D13<FMC>, PD14: D14<FMC>, PD15: D15<FMC>, PNBL0: NBL0<FMC>, PNBL1:
            NBL1<FMC>, PSDCKEn: $ckeN<FMC>, PSDCLK: SDCLK<FMC>, PSDNCAS:
            SDNCAS<FMC>, PSDNEn: $neN<FMC>, PSDNRAS: SDNRAS<FMC>, PSDNWE: SDNWE<FMC> {
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
            impl<FMC, PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10,
            PA11, PBA0, $($pba1,)* PD0, PD1, PD2, PD3, PD4, PD5, PD6, PD7, PD8,
            PD9, PD10, PD11, PD12, PD13, PD14, PD15, PD16, PD17, PD18, PD19,
            PD20, PD21, PD22, PD23, PD24, PD25, PD26, PD27, PD28, PD29, PD30,
            PD31, PNBL0, PNBL1, PNBL2, PNBL3, PSDCKEn, PSDCLK, PSDNCAS,
            PSDNEn, PSDNRAS, PSDNWE>
                PinsSdram<FMC>
                for $pins<(PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10,
                     PA11, PBA0, $($pba1,)* PD0, PD1, PD2, PD3, PD4, PD5, PD6, PD7,
                     PD8, PD9, PD10, PD11, PD12, PD13, PD14, PD15, PD16, PD17,
                     PD18, PD19, PD20, PD21, PD22, PD23, PD24, PD25, PD26, PD27,
                     PD28, PD29, PD30, PD31, PNBL0, PNBL1, PNBL2, PNBL3, PSDCKEn,
                     PSDCLK, PSDNCAS, PSDNEn, PSDNRAS, PSDNWE)>
            where PA0: A0<FMC>, PA1: A1<FMC>, PA2: A2<FMC>, PA3: A3<FMC>, PA4:
            A4<FMC>, PA5: A5<FMC>, PA6: A6<FMC>, PA7: A7<FMC>, PA8: A8<FMC>, PA9:
            A9<FMC>, PA10: A10<FMC>, PA11: A11<FMC>, PBA0: BA0<FMC>,
            $($pba1:$ba1<FMC>,)*
            PD0: D0<FMC>, PD1: D1<FMC>, PD2: D2<FMC>, PD3: D3<FMC>, PD4:
            D4<FMC>, PD5: D5<FMC>, PD6: D6<FMC>, PD7: D7<FMC>, PD8: D8<FMC>, PD9:
            D9<FMC>, PD10: D10<FMC>, PD11: D11<FMC>, PD12: D12<FMC>, PD13:
            D13<FMC>, PD14: D14<FMC>, PD15: D15<FMC>, PD16: D16<FMC>, PD17:
            D17<FMC>, PD18: D18<FMC>, PD19: D19<FMC>, PD20: D20<FMC>, PD21:
            D21<FMC>, PD22: D22<FMC>, PD23: D23<FMC>, PD24: D24<FMC>, PD25:
            D25<FMC>, PD26: D26<FMC>, PD27: D27<FMC>, PD28: D28<FMC>, PD29:
            D29<FMC>, PD30: D30<FMC>, PD31: D31<FMC>, PNBL0: NBL0<FMC>, PNBL1:
            NBL1<FMC>, PNBL2: NBL2<FMC>, PNBL3: NBL3<FMC>, PSDCKEn: $ckeN<FMC>,
                  PSDCLK: SDCLK<FMC>, PSDNCAS: SDNCAS<FMC>, PSDNEn: $neN<FMC>, PSDNRAS:
            SDNRAS<FMC>, PSDNWE: SDNWE<FMC> {
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

pub trait A0<FMC> {}
pub trait A1<FMC> {}
pub trait A10<FMC> {}
pub trait A11<FMC> {}
pub trait A12<FMC> {}
pub trait A13<FMC> {}
pub trait A14<FMC> {}
pub trait A15<FMC> {}
pub trait A16<FMC> {}
pub trait A17<FMC> {}
pub trait A18<FMC> {}
pub trait A19<FMC> {}
pub trait A2<FMC> {}
pub trait A20<FMC> {}
pub trait A21<FMC> {}
pub trait A22<FMC> {}
pub trait A23<FMC> {}
pub trait A24<FMC> {}
pub trait A25<FMC> {}
pub trait A3<FMC> {}
pub trait A4<FMC> {}
pub trait A5<FMC> {}
pub trait A6<FMC> {}
pub trait A7<FMC> {}
pub trait A8<FMC> {}
pub trait A9<FMC> {}
pub trait BA0<FMC> {}
pub trait BA1<FMC> {}
pub trait CLK<FMC> {}
pub trait D0<FMC> {}
pub trait D1<FMC> {}
pub trait D10<FMC> {}
pub trait D11<FMC> {}
pub trait D12<FMC> {}
pub trait D13<FMC> {}
pub trait D14<FMC> {}
pub trait D15<FMC> {}
pub trait D16<FMC> {}
pub trait D17<FMC> {}
pub trait D18<FMC> {}
pub trait D19<FMC> {}
pub trait D2<FMC> {}
pub trait D20<FMC> {}
pub trait D21<FMC> {}
pub trait D22<FMC> {}
pub trait D23<FMC> {}
pub trait D24<FMC> {}
pub trait D25<FMC> {}
pub trait D26<FMC> {}
pub trait D27<FMC> {}
pub trait D28<FMC> {}
pub trait D29<FMC> {}
pub trait D3<FMC> {}
pub trait D30<FMC> {}
pub trait D31<FMC> {}
pub trait D4<FMC> {}
pub trait D5<FMC> {}
pub trait D6<FMC> {}
pub trait D7<FMC> {}
pub trait D8<FMC> {}
pub trait D9<FMC> {}
pub trait DA0<FMC> {}
pub trait DA1<FMC> {}
pub trait DA10<FMC> {}
pub trait DA11<FMC> {}
pub trait DA12<FMC> {}
pub trait DA13<FMC> {}
pub trait DA14<FMC> {}
pub trait DA15<FMC> {}
pub trait DA2<FMC> {}
pub trait DA3<FMC> {}
pub trait DA4<FMC> {}
pub trait DA5<FMC> {}
pub trait DA6<FMC> {}
pub trait DA7<FMC> {}
pub trait DA8<FMC> {}
pub trait DA9<FMC> {}
pub trait INT<FMC> {}
pub trait NBL0<FMC> {}
pub trait NBL1<FMC> {}
pub trait NBL2<FMC> {}
pub trait NBL3<FMC> {}
pub trait NCE<FMC> {}
pub trait NE1<FMC> {}
pub trait NE2<FMC> {}
pub trait NE3<FMC> {}
pub trait NE4<FMC> {}
pub trait NL<FMC> {}
pub trait NOE<FMC> {}
pub trait NWAIT<FMC> {}
pub trait NWE<FMC> {}
pub trait SDCKE0<FMC> {}
pub trait SDCKE1<FMC> {}
pub trait SDCLK<FMC> {}
pub trait SDNCAS<FMC> {}
pub trait SDNE0<FMC> {}
pub trait SDNE1<FMC> {}
pub trait SDNRAS<FMC> {}
pub trait SDNWE<FMC> {}
impl Fmc {
    /// New FMC instance
    pub fn new(fmc: FMC, // , rec_fmc: rec::Fmc
    ) -> Self {
        // Enable clock and reset
        // let rec_fmc = rec_fmc.enable().reset();
        // let clk_sel = rec_fmc.get_kernel_clk_mux();

        Fmc { fmc } //, clk_sel }
    }

    // /// Current kernel clock (`fmc_ker_ck`)
    // pub fn get_ker_clk(&self, clocks: CoreClocks) -> Option<Hertz> {
    //     match self.clk_sel {
    //         rec::FmcClkSel::RCC_HCLK3 => Some(clocks.hclk()),
    //         rec::FmcClkSel::PLL1_Q => clocks.pll1_q_ck(),
    //         rec::FmcClkSel::PLL2_R => clocks.pll2_r_ck(),
    //         rec::FmcClkSel::PER => clocks.per_ck(),
    //     }
    // }

    /// Enable FMC controller
    pub(crate) fn enable(&mut self) {
        // The FMCEN bit of the FMC_BCR2..4 registers is don’t
        // care. It is only enabled through the FMC_BCR1 register.

        // H7 only?
        //self.fmc.bcr1.modify(|_, w| w.fmcen().set_bit());
    }
}
