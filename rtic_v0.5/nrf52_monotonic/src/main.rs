//! In here all hardware dependent code is kept, and to run the independent parts the firmware crate
//! is called.

#![no_main]
#![no_std]

mod monotonic_nrf52;

use monotonic_nrf52::*;
use nrf52832_hal as _;
// use panic_halt as _;
use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use rtic::app;

#[app(device = nrf52832_hal::target, peripherals = true, monotonic = crate::monotonic_nrf52::Tim1)]
const APP: () = {
    #[init (spawn = [task1])]
    fn init(cx: init::Context) {
        Tim1::initialize(cx.device.TIMER1);
        hprintln!("init").ok();
        cx.spawn.task1().ok();
    }

    #[task(schedule = [task1])]
    fn task1(cx: task1::Context) {
        hprintln!("here").ok();
        cx.schedule.task1(cx.scheduled + 2000.millis()).ok();
    }

    extern "C" {
        fn SWI0_EGU0();
    }
};
