//! HAL for external SDRAM

use core::cmp;
use core::convert::TryInto;
use core::marker::PhantomData;

use embedded_hal::blocking::delay::DelayUs;

use crate::fmc::{FmcBank, FmcRegisters, PinsSdram};
use crate::FmcPeripheral;

use crate::ral::{fmc, modify_reg, write_reg};

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
    pub mode_register_to_active: u32,
    /// Delay from releasing self refresh to next command
    pub exit_self_refresh: u32,
    /// Delay between an ACTIVATE and a PRECHARGE command
    pub active_to_precharge: u32,
    /// Auto refresh command duration
    pub row_cycle: u32,
    /// Delay between a PRECHARGE command and another command
    pub row_precharge: u32,
    /// Delay between an ACTIVATE command and READ/WRITE command
    pub row_to_column: u32,
}

pub trait SdramChip {
    const MODE_REGISTER: u16;
    const CONFIG: FmcSdramConfiguration;
    const TIMING: FmcSdramTiming;
}

/// SDRAM Controller
#[allow(missing_debug_implementations)]
pub struct Sdram<IC, PINS, FMC> {
    /// FMC pins
    _pins: PhantomData<PINS>,
    /// Parameters for the SDRAM IC
    _chip: PhantomData<IC>,
    /// FMC peripheral
    fmc: FMC,
    /// Register access
    regs: FmcRegisters,
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

/// Like `modfiy_reg`, but applies to bank 1 or 2 based on a varaiable
macro_rules! modify_reg_banked {
    ( $periph:path, $instance:expr, $bank:expr, $reg1:ident, $reg2:ident, $( $field:ident : $value:expr ),+ ) => {{
        match $bank {
            1 => modify_reg!( $periph, $instance, $reg1, $( $field : $value ),*),
            2 => modify_reg!( $periph, $instance, $reg2, $( $field : $value ),*),
            _ => panic!(),
        }
    }};
}

impl<IC, PINS, FMC: FmcPeripheral> Sdram<IC, PINS, FMC>
where
    IC: SdramChip,
    PINS: PinsSdram,
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
    pub fn new(fmc: FMC, _pins: PINS, _chip: IC) -> Self {
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
            _pins: PhantomData,
            _chip: PhantomData,
            fmc,
            regs: FmcRegisters::new::<FMC>(),
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
    pub unsafe fn new_unchecked(fmc: FMC, _chip: IC) -> Self {
        Sdram {
            _pins: PhantomData,
            _chip: PhantomData,
            fmc,
            regs: FmcRegisters::new::<FMC>(),
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
    pub fn init<D>(&mut self, delay: &mut D) -> *mut u32
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

        // Calcuate SD clock
        let sd_clock_hz = {
            let fmc_source_ck_hz = self.fmc.source_clock_hz();

            fmc_source_ck_hz / IC::CONFIG.sd_clock_divide as u32
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
            // Enable memory controller registers
            FMC::enable();

            // Program device features and timing
            self.set_features_timings(
                PINS::EXTERNAL_BANK,
                IC::CONFIG,
                IC::TIMING,
            );

            // Enable memory controller
            FMC::memory_controller_enable();

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

            modify_reg!(
                fmc,
                self.regs.global(),
                SDRTR,
                COUNT: refresh_counter_top as u32
            );
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
    /// banks and different configurations is unsafe.
    ///
    /// For example, see RM0433 rev 7 Section 22.9.3
    unsafe fn set_features_timings(
        &mut self,
        sdram_bank: u8,
        config: FmcSdramConfiguration,
        timing: FmcSdramTiming,
    ) {
        // Features ---- SDCR REGISTER

        // CAS latency 1 ~ 3 cycles
        assert!(
            config.cas_latency >= 1 && config.cas_latency <= 3,
            "Impossible configuration for FMC Controller"
        );

        // Row Bits: 11 ~ 13
        assert!(
            config.row_bits >= 11 && config.row_bits <= 13,
            "Impossible configuration for FMC Controller"
        );

        // Column bits: 8 ~ 11
        assert!(
            config.column_bits >= 8 && config.column_bits <= 11,
            "Impossible configuration for FMC Controller"
        );

        // Read Pipe Delay Cycles 0 ~ 2
        assert!(
            config.read_pipe_delay_cycles <= 2,
            "Impossible configuration for FMC Controller"
        );

        // Common settings written to SDCR1 only
        modify_reg!(fmc, self.regs.global(), SDCR1,
                    RPIPE: config.read_pipe_delay_cycles as u32,
                    RBURST: config.read_burst as u32,
                    SDCLK: config.sd_clock_divide as u32);

        modify_reg_banked!(fmc, self.regs.global(),
                           sdram_bank, SDCR1, SDCR2,
                           // fields
                           WP: config.write_protection as u32,
                           CAS: config.cas_latency as u32,
                           NB:
                           match config.internal_banks {
                               2 => 0,
                               4 => 1,
                               _ => {
                                   panic!("Impossible configuration for FMC Controller")
                               }
                           },
                           MWID:
                           match config.memory_data_width {
                               8 => 0,
                               16 => 1,
                               32 => 2,
                               _ => {
                                   panic!("Impossible configuration for FMC Controller")
                               }
                           },
                           NR: config.row_bits as u32 - 11,
                           NC: config.column_bits as u32 - 8);

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
        modify_reg!(fmc, self.regs.global(), SDTR1,
                    TRC: timing.row_cycle - 1,
                    TRP: timing.row_precharge - 1
        );
        modify_reg_banked!(fmc, self.regs.global(),
                           sdram_bank, SDTR1, SDTR2,
                           // fields
                           TRCD: timing.row_to_column - 1,
                           TWR: write_recovery - 1,
                           TRAS: minimum_self_refresh - 1,
                           TXSR: timing.exit_self_refresh - 1,
                           TMRD: timing.mode_register_to_active - 1
        );
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
            Bank1 => (1, 0),
            Bank2 => (0, 1),
            Both => (1, 1),
        };

        // Write to SDCMR
        write_reg!(
            fmc,
            self.regs.global(),
            SDCMR,
            MRD: mode_reg as u32,
            NRFS: number_refresh as u32,
            CTB1: b1,
            CTB2: b2,
            MODE: cmd
        );
    }
}
