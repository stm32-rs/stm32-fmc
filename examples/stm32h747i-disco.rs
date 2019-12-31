#![no_main]
#![no_std]

use core::mem;
use core::slice;

extern crate cortex_m_rt as rt;
use core::sync::atomic::{AtomicU32, Ordering};
use rt::{entry, exception};

extern crate cortex_m;
extern crate panic_itm;

#[macro_use]
extern crate log;

extern crate stm32h7_fmc;
use stm32h7_fmc::is42s32800g_6;

use stm32h7xx_hal::gpio::Speed;
use stm32h7xx_hal::hal::digital::v2::OutputPin;
use stm32h7xx_hal::rcc::CoreClocks;
use stm32h7xx_hal::{prelude::*, stm32};

use cortex_m_log::log::{trick_init, Logger};
use cortex_m_log::{
    destination::Itm, printer::itm::InterruptSync as InterruptSyncItm,
};

/// Configure SYSTICK for 1ms timebase
fn systick_init(syst: &mut stm32::SYST, clocks: CoreClocks) {
    let c_ck_mhz = clocks.c_ck().0 / 1_000_000;

    let syst_calib = 0x3E8;

    syst.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    syst.set_reload((syst_calib * c_ck_mhz) - 1);
    syst.enable_interrupt();
    syst.enable_counter();
}

/// Configure MPU for external SDRAM
///
/// Memory address in location will be 32-byte aligned.
///
/// # Panics
///
/// Function will panic if `size` is not a power of 2. Function
/// will panic if `size` is not at least 32 bytes.
fn mpu_sdram_init(
    mpu: &mut stm32::MPU,
    scb: &mut stm32::SCB,
    location: *mut u32,
    size: usize,
) {
    /// Refer to ARM®v7-M Architecture Reference Manual ARM DDI 0403
    /// Version E.b Section B3.5
    const MEMFAULTENA: u32 = 1 << 16;

    unsafe {
        /* Make sure outstanding transfers are done */
        cortex_m::asm::dmb();

        scb.shcsr.modify(|r| r & !MEMFAULTENA);

        /* Disable the MPU and clear the control register*/
        mpu.ctrl.write(0);
    }

    const REGION_NUMBER0: u32 = 0x00;
    const REGION_FULL_ACCESS: u32 = 0x03;
    const REGION_ENABLE: u32 = 0x01;

    assert_eq!(
        size & (size - 1),
        0,
        "SDRAM memory region size must be a power of 2"
    );
    assert_eq!(
        size & 0x1F,
        0,
        "SDRAM memory region size must be 32 bytes or more"
    );
    fn log2minus1(sz: u32) -> u32 {
        for x in 5..=31 {
            if sz == (1 << x) {
                return x - 1;
            }
        }
        panic!("Unknown SDRAM memory region size!");
    }

    info!("SDRAM Memory Size 0x{:x}", log2minus1(size as u32));

    // Configure region 0
    //
    // Strongly ordered
    unsafe {
        mpu.rnr.write(REGION_NUMBER0);
        mpu.rbar.write((location as u32) & !0x1F);
        mpu.rasr.write(
            (REGION_FULL_ACCESS << 24)
                | (log2minus1(size as u32) << 1)
                | REGION_ENABLE,
        );
    }

    const MPU_ENABLE: u32 = 0x01;
    const MPU_DEFAULT_MMAP_FOR_PRIVILEGED: u32 = 0x04;

    // Enable
    unsafe {
        mpu.ctrl
            .modify(|r| r | MPU_DEFAULT_MMAP_FOR_PRIVILEGED | MPU_ENABLE);

        scb.shcsr.modify(|r| r | MEMFAULTENA);

        // Ensure MPU settings take effect
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
    }
}

/// ======================================================================
/// Entry point
/// ======================================================================

/// TIME is an atomic u32 that counts milliseconds. Although not used
/// here, it is very useful to have for network protocols
static TIME: AtomicU32 = AtomicU32::new(0);

/// Configure pins for the FMC controller
macro_rules! fmc_pins {
    ($($pin:expr),*) => {
        (
            $(
                $pin.into_push_pull_output()
                    .set_speed(Speed::VeryHigh)
                    .into_alternate_af12()
                    .internal_pull_up(true)
            ),*
        )
    };
}

// the program entry point
#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let mut cp = stm32::CorePeripherals::take().unwrap();

    // Initialise logging...
    let logger = Logger {
        inner: InterruptSyncItm::new(Itm::new(cp.ITM)),
        level: log::LevelFilter::Trace,
    };
    unsafe {
        let _ = trick_init(&logger);
    }

    // Initialise power...
    let pwr = dp.PWR.constrain();
    let vos = pwr.freeze();

    // Initialise clocks...
    let rcc = dp.RCC.constrain();
    let ccdr = rcc
        .use_hse(25.mhz()) // XTAL X2
        .sys_ck(200.mhz())
        .hclk(200.mhz())
        .pll1_r_ck(100.mhz()) // for TRACECK
        .freeze(vos, &dp.SYSCFG);

    // Get the delay provider.
    let mut delay = cp.SYST.delay(ccdr.clocks);

    // Initialise system...
    cp.SCB.invalidate_icache();
    cp.SCB.enable_icache();
    cp.SCB.enable_dcache(&mut cp.CPUID);
    cp.DWT.enable_cycle_counter();

    // Initialise IO...
    let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let gpiof = dp.GPIOF.split(ccdr.peripheral.GPIOF);
    let gpiog = dp.GPIOG.split(ccdr.peripheral.GPIOG);
    let gpioh = dp.GPIOH.split(ccdr.peripheral.GPIOH);
    let gpioi = dp.GPIOI.split(ccdr.peripheral.GPIOI);

    let mut lcd_led = gpioi.pi13.into_push_pull_output(); // LED2
    lcd_led.set_low().ok();

    // Initialise SDRAM...
    let fmc_io = stm32h7_fmc::PinsSdramBank2(fmc_pins! {
        // A0-A11
        gpiof.pf0, gpiof.pf1, gpiof.pf2, gpiof.pf3,
        gpiof.pf4, gpiof.pf5, gpiof.pf12, gpiof.pf13,
        gpiof.pf14, gpiof.pf15, gpiog.pg0, gpiog.pg1,
        // BA0-BA1
        gpiog.pg4, gpiog.pg5,
        // D0-D31
        gpiod.pd14, gpiod.pd15, gpiod.pd0, gpiod.pd1,
        gpioe.pe7, gpioe.pe8, gpioe.pe9, gpioe.pe10,
        gpioe.pe11, gpioe.pe12, gpioe.pe13, gpioe.pe14,
        gpioe.pe15, gpiod.pd8, gpiod.pd9, gpiod.pd10,
        gpioh.ph8, gpioh.ph9, gpioh.ph10, gpioh.ph11,
        gpioh.ph12, gpioh.ph13, gpioh.ph14, gpioh.ph15,
        gpioi.pi0, gpioi.pi1, gpioi.pi2, gpioi.pi3,
        gpioi.pi6, gpioi.pi7, gpioi.pi9, gpioi.pi10,
        // NBL0 - NBL3
        gpioe.pe0, gpioe.pe1, gpioi.pi4, gpioi.pi5,
        gpioh.ph7,              // SDCKE1
        gpiog.pg8,              // SDCLK
        gpiog.pg15,             // SDNCAS
        gpioh.ph6,              // SDNE1 (!CS)
        gpiof.pf11,             // SDRAS
        gpioh.ph5               // SDNWE
    });

    let mut sdram = stm32h7_fmc::Sdram::new(
        dp.FMC,
        ccdr.peripheral.FMC,
        fmc_io,
        is42s32800g_6::Is42s32800g {},
    );

    // Initialise controller and SDRAM
    let ram = unsafe {
        let ram_ptr: *mut u32 = sdram.init(&mut delay, ccdr.clocks);
        let ram_size_bytes = 32 * 1024 * 1024;

        // MPU config for SDRAM write-through
        mpu_sdram_init(&mut cp.MPU, &mut cp.SCB, ram_ptr, ram_size_bytes);

        info!("");
        info!("");
        info!("Initialised MPU...");

        slice::from_raw_parts_mut(
            ram_ptr,
            ram_size_bytes / mem::size_of::<u32>(),
        )
    };

    info!("Initialised SDRAM...");

    // ----------------------------------------------------------
    // Begin periodic tasks

    systick_init(&mut delay.free(), ccdr.clocks);
    unsafe {
        cp.SCB.shpr[15 - 4].write(128);
    } // systick exception priority

    // ----------------------------------------------------------
    // Main application loop
    let len = 8 * 1024 * 64; //1024;

    for a in 0..len {
        ram[a] = a as u32;
    }

    info!("");

    cortex_m::asm::dsb();

    for a in 0..len {
        assert_eq!(a as u32, ram[a]);
    }

    info!("SDRAM checked ok!");

    // Worst-case check sequence for EMC
    let mask = (ram.len() / 4) - 1;
    let address = 0x5555_5555 & mask;
    let data = 0xAAAA_AAAA;
    ram[address] = data;
    ram[!address & mask] = !data;

    loop {
        ram[address] = data;
        cortex_m::asm::dsb();

        assert_eq!(ram[!address & mask], !data);
        cortex_m::asm::dsb();

        ram[!address & mask] = !data;
        cortex_m::asm::dsb();

        assert_eq!(ram[address], data);
        cortex_m::asm::dsb();
    }
}

#[exception]
fn SysTick() {
    TIME.fetch_add(1, Ordering::Relaxed);
}

#[exception]
fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
