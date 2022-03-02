//! In here all hardware dependent code is kept, and to run the independent parts the firmware crate
//! is called.

#![no_main]
#![no_std]

use crate::monotonic_nrf52::MonoTimer;
use fugit::{self, ExtU32};
use nrf52832_hal as _;
use panic_rtt_target as _;
use rtic::app;
use rtt_target::{rprintln, rtt_init_print};

mod monotonic_nrf52;

#[app(device = nrf52832_hal::pac, peripherals = true, dispatchers = [SWI0_EGU0])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[monotonic(binds = TIMER1, default = true)]
    type Tonic = MonoTimer<nrf52832_hal::pac::TIMER1>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mono = MonoTimer::new(cx.device.TIMER1);

        rtt_init_print!();
        rprintln!("init");

        task1::spawn().ok();

        (Shared {}, Local {}, init::Monotonics(mono))
    }

    #[task]
    fn task1(_cx: task1::Context) {
        rprintln!("task1");
        task1::spawn_after(2000.millis()).ok();
    }
}
