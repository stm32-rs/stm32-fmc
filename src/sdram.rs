//! HAL for external SDRAM

use core::cmp;
use core::convert::TryInto;
use core::marker::PhantomData;

use crate::hal::blocking::delay::DelayUs;
use crate::stm32;

use crate::fmc::{Fmc, FmcBank, PinsSdram};

/// FMC SDRAM Configuration Structure definition
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FmcSdramConfiguration {
    /// Number of bits of column address
    pub column_bits: u8,
    /// Number of bits of column address
    pub row_bits: u8,
    /// Memory device width
    pub memory_data_width: u8,
    /// Number of the device's internal banks
    pub internal_banks: u8,
    /// SDRAM CAS latency in number of memory clock cycles
    pub cas_latency: u8,
    /// Enables the SDRAM device to be accessed in write mode
    pub write_protection: bool,
    /// SDRAM clock divider
    pub sd_clock_divide: u8,
    /// This bit enable the SDRAM controller to anticipate the next read
    pub read_burst: bool,
    /// Delay in system clock cycles on read data path
    pub read_pipe_delay_cycles: u8,
}

/// FMC SDRAM Timing parameters structure definition
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FmcSdramTiming {
    /// Time between applying a valid clock and any command other than
    /// COMMAND INHIBIT or NOP
    pub startup_delay_ns: u32,
    /// Maximum SD clock frequency to make timing
    pub max_sd_clock_hz: u32,
    /// Period between refresh cycles in nanoseconds
    pub refresh_period_ns: u32,
    /// Delay between a LOAD MODE register command and an ACTIVATE command
    pub mode_register_to_active: u8,
    /// Delay from releasing self refresh to next command
    pub exit_self_refresh: u8,
    /// Delay between an ACTIVATE and a PRECHARGE command
    pub active_to_precharge: u8,
    /// Auto refresh command duration
    pub row_cycle: u8,
    /// Delay between a PRECHARGE command and another command
    pub row_precharge: u8,
    /// Delay between an ACTIVATE command and READ/WRITE command
    pub row_to_column: u8,
}

pub trait SdramChip {
    const MODE_REGISTER: u16;
    const CONFIG: FmcSdramConfiguration;
    const TIMING: FmcSdramTiming;
}

/// SDRAM Controller
#[allow(missing_debug_implementations)]
pub struct Sdram<IC, PINS> {
    mem: Fmc,
    /// FMC pins
    _pins: PhantomData<PINS>,
    /// Parameters for the SDRAM IC
    _chip: PhantomData<IC>,
}

/// SDRAM Commands
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(unused)]
enum SdramCommand {
    NormalMode,
    ClkEnable,
    Pall,
    Autorefresh(u8),
    LoadMode(u16),
    Selfrefresh,
    Powerdown,
}
/// Target bank for SDRAM commands
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(unused)]
enum SdramTargetBank {
    Bank1,
    Bank2,
    Both,
}

impl<IC, PINS> Sdram<IC, PINS>
where
    IC: SdramChip,
    PINS: PinsSdram<stm32::FMC>,
{
    /// New SDRAM instance
    ///
    /// `_pins` must be a set of pins connecting to an SDRAM on the
    /// FMC controller. This is currently implemented for the types
    /// [`PinsSdramBank1`](struct.PinsSdramBank1.html) and
    /// [`PinsSdramBank2`](struct.PinsSdramBank2.html)
    ///
    /// # Panics
    ///
    /// * Panics if there are not enough address lines in `PINS` to
    /// access the whole SDRAM
    ///
    /// * Panics if there are not enough bank address lines in `PINS`
    /// to access the whole SDRAM
    pub fn new(
        fmc: stm32::FMC,
        //rec_fmc: rec::Fmc,
        _pins: PINS,
        _chip: IC,
    ) -> Self {
        assert!(
            PINS::ADDRESS_LINES >= IC::CONFIG.row_bits,
            "Not enough address pins to access all SDRAM rows"
        );
        assert!(
            PINS::ADDRESS_LINES >= IC::CONFIG.column_bits,
            "Not enough address pins to access all SDRAM colums"
        );
        assert!(
            PINS::NUMBER_INTERNAL_BANKS >= IC::CONFIG.internal_banks,
            "Not enough bank address pins to access all internal banks"
        );

        Sdram {
            mem: Fmc::new(fmc),
            _pins: PhantomData,
            _chip: PhantomData,
        }
    }

    /// New SDRAM instance
    ///
    /// `_pins` must be a set of pins connecting to an SDRAM on the
    /// FMC controller. This is currently implemented for the types
    /// [`PinsSdramBank1`](struct.PinsSdramBank1.html) and
    /// [`PinsSdramBank2`](struct.PinsSdramBank2.html)
    ///
    /// # Safety
    ///
    /// The pins are not checked against the requirements for this
    /// SDRAM chip. So you may be able to initialise a SDRAM without
    /// enough pins to access the whole memory
    pub unsafe fn new_unchecked(
        fmc: stm32::FMC,
        // rec_fmc: rec::Fmc,
        _pins: PINS,
        _chip: IC,
    ) -> Self {
        Sdram {
            mem: Fmc::new(fmc),
            _pins: PhantomData,
            _chip: PhantomData,
        }
    }

    /// Initialise SDRAM instance. Delay is used to wait the SDRAM
    /// powerup delay
    ///
    /// Returns a raw pointer to the memory-mapped SDRAM block
    ///
    /// # Panics
    ///
    /// * Panics if the FMC kernel clock `fmc_ker_ck` is not running
    ///
    /// * Panics if any setting in `IC::CONFIG` cannot be achieved
    ///
    /// * Panics if the FMC kernal clock `fmc_ker_ck` is too fast for
    /// maximum SD clock in `IC::TIMING`
    pub fn init<D>(
        &mut self,
        delay: &mut D,
        // core_clocks: CoreClocks,
    ) -> *mut u32
    where
        D: DelayUs<u8>,
    {
        use SdramCommand::*;
        use SdramTargetBank::*;

        // Select bank
        let bank = match PINS::EXTERNAL_BANK {
            1 => Bank1,
            2 => Bank2,
            _ => unimplemented!(),
        };

        // Clock divider 2 ~ 3
        assert!(
            IC::CONFIG.sd_clock_divide >= 2 && IC::CONFIG.sd_clock_divide <= 3,
            "SD clock divider is invalid!"
        );

        // Calcuate SD clock from the current `fmc_ker_ck`
        let sd_clock_hz = {
            let fmc_ker_ck_hz = 100_000_000; // self
                                             // .mem
                                             // .get_ker_clk(core_clocks)
                                             // .expect("FMC kernel clock is not running!")
                                             // .0;
            fmc_ker_ck_hz / IC::CONFIG.sd_clock_divide as u32
        };
        // Check that the SD clock is acceptable
        assert!(
            sd_clock_hz <= IC::TIMING.max_sd_clock_hz,
            "FMC kernel clock is too fast for the SD
                 clock period of the SDRAM!"
        );

        fmc_trace!(
            "FMC clock {:?} (Max {:?})",
            sd_clock_hz,
            IC::TIMING.max_sd_clock_hz
        );

        unsafe {
            // Program device features and timing
            self.set_features_timings(
                PINS::EXTERNAL_BANK,
                IC::CONFIG,
                IC::TIMING,
            );

            // Enable controller
            self.mem.enable();

            // Step 1: Send a clock configuration enable command
            self.send_command(ClkEnable, bank);

            // Step 2: SDRAM powerup delay
            let startup_delay_us = (IC::TIMING.startup_delay_ns + 999) / 1000;
            delay.delay_us(startup_delay_us.try_into().unwrap());

            // Step 3: Send a PALL (precharge all) command
            self.send_command(Pall, bank);

            // Step 4: Send eight auto refresh commands
            self.send_command(Autorefresh(8), bank);

            // Step 5: Program the SDRAM's mode register
            self.send_command(LoadMode(IC::MODE_REGISTER), bank);

            // Step 6: Set the refresh rate counter
            // period (ns) * frequency (hz) / 10^9 = count
            let refresh_counter_top = ((IC::TIMING.refresh_period_ns as u64
                * sd_clock_hz as u64)
                / 1_000_000_000)
                - 20;
            assert!(
                refresh_counter_top >= 41 && refresh_counter_top < (1 << 13),
                "Impossible configuration for H7 FMC Controller"
            );
            self.mem
                .fmc
                .sdrtr
                .modify(|_, w| w.count().bits(refresh_counter_top as u16));
        }

        // Memory now initialised. Return base address
        match PINS::EXTERNAL_BANK {
            1 => FmcBank::Bank5.ptr(),
            2 => FmcBank::Bank6.ptr(),
            _ => unimplemented!(),
        }
    }

    /// Program memory device features and timings
    ///
    /// # Safety
    ///
    /// Some settings are common between both
    /// banks. Calling this function mutliple times with different
    /// banks and different configurations is unsafe. Refer to
    /// RM0433 Section 21.9 / RM0399 Section 23.9
    unsafe fn set_features_timings(
        &mut self,
        sdram_bank: u8,
        config: FmcSdramConfiguration,
        timing: FmcSdramTiming,
    ) {
        // SDRAM Controller/Timing registers
        // let sd = match sdram_bank {
        //     1 => self.mem.fmc.sdbank1(),
        //     2 => self.mem.fmc.sdbank2(),
        //     _ => panic!(),
        // };

        // Features ---- SDCR REGISTER

        // CAS latency 1 ~ 3 cycles
        assert!(
            config.cas_latency >= 1 && config.cas_latency <= 3,
            "Impossible configuration for H7 FMC Controller"
        );

        // Row Bits: 11 ~ 13
        assert!(
            config.row_bits >= 11 && config.row_bits <= 13,
            "Impossible configuration for H7 FMC Controller"
        );

        // Column bits: 8 ~ 11
        assert!(
            config.column_bits >= 8 && config.column_bits <= 11,
            "Impossible configuration for H7 FMC Controller"
        );

        // Read Pipe Delay Cycles 0 ~ 2
        assert!(
            config.read_pipe_delay_cycles <= 2,
            "Impossible configuration for H7 FMC Controller"
        );

        // Common settings written to SDCR1 only
        self.mem.fmc.sdcr1.modify(|_, w| {
            w.rpipe()
                .bits(config.read_pipe_delay_cycles)
                .rburst()
                .bit(config.read_burst)
                .sdclk()
                .bits(config.sd_clock_divide)
        });
        self.mem.fmc.sdcr1.modify(|_, w| {
            w.wp()
                .bit(config.write_protection)
                .cas()
                .bits(config.cas_latency)
                .nb()
                .bit(match config.internal_banks {
                    2 => false,
                    4 => true,
                    _ => {
                        panic!("Impossible configuration for H7 FMC Controller")
                    }
                })
                .mwid()
                .bits(match config.memory_data_width {
                    8 => 0,
                    16 => 1,
                    32 => 2,
                    _ => {
                        panic!("Impossible configuration for H7 FMC Controller")
                    }
                })
                .nr()
                .bits(config.row_bits - 11)
                .nc()
                .bits(config.column_bits - 8)
        });

        // Timing ---- SDTR REGISTER

        // Self refresh >= ACTIVE to PRECHARGE
        let minimum_self_refresh = timing.active_to_precharge;

        // Write recovery - Self refresh
        let write_recovery_self_refresh =
            minimum_self_refresh - timing.row_to_column;
        // Write recovery - WRITE command to PRECHARGE command
        let write_recovery_row_cycle =
            timing.row_cycle - timing.row_to_column - timing.row_precharge;
        let write_recovery =
            cmp::max(write_recovery_self_refresh, write_recovery_row_cycle);

        // Common seting written to SDTR1 only
        self.mem.fmc.sdtr1.modify(|_, w| {
            w.trc()
                .bits(timing.row_cycle - 1)
                .trp()
                .bits(timing.row_precharge - 1)
        });
        self.mem.fmc.sdtr1.modify(|_, w| {
            w.trcd()
                .bits(timing.row_to_column - 1)
                .twr()
                .bits(write_recovery - 1)
                .tras()
                .bits(minimum_self_refresh - 1)
                .txsr()
                .bits(timing.exit_self_refresh - 1)
                .tmrd()
                .bits(timing.mode_register_to_active - 1)
        });
    }

    /// Send command to SDRAM
    unsafe fn send_command(
        &mut self,
        mode: SdramCommand,
        target: SdramTargetBank,
    ) {
        use SdramCommand::*;
        use SdramTargetBank::*;

        // Command
        let (cmd, number_refresh, mode_reg) = match mode {
            NormalMode => (0x00, 1, 0),
            ClkEnable => (0x01, 1, 0),
            Pall => (0x02, 1, 0),
            Autorefresh(a) => (0x03, a, 0), // Autorefresh
            LoadMode(mr) => (0x04, 1, mr),  // Mode register
            Selfrefresh => (0x05, 1, 0),
            Powerdown => (0x06, 1, 0),
        };
        // Bank for issuing command
        let (b1, b2) = match target {
            Bank1 => (true, false),
            Bank2 => (false, true),
            Both => (true, true),
        };

        // Write to SDCMR
        self.mem.fmc.sdcmr.modify(|_, w| {
            w.mrd()
                .bits(mode_reg)
                .nrfs()
                .bits(number_refresh)
                .ctb1()
                .bit(b1)
                .ctb2()
                .bit(b2)
                .mode()
                .bits(cmd)
        });
    }
}
