#![no_main]
#![no_std]

use panic_rtt_target as _;

mod monotonic_stm32l0;
use core::fmt::Write;
use embedded_time::rate::Baud;
use monotonic_stm32l0::{Duration, Instant, Tim6Monotonic, U16Ext};
use rtic::app;
use rtt_target::{rprintln, rtt_init_print};
use stm32l0xx_hal::{pac, prelude::*, rcc::Config, serial};

const INTERVAL_MS: u16 = 500;

#[app(
    device = stm32l0xx_hal::pac,
    peripherals = true,
    dispatchers = [SPI1],
)]
mod app {
    use super::*;

    // Setting this monotonic as the default
    // enables the shorthand fizzbuzz::spawn_after
    // without having to specify `Mono` as fizzbuzz::Mono::spawn_after(
    #[monotonic(binds = TIM6, default = true)]
    type Mono = Tim6Monotonic;

    #[local]
    struct Local {
        /// Serial debug output
        // serial: serial::Serial<pac::USART1>,

        /// Timer interval
        interval: Duration,

        /// Counter
        counter: usize,
    }

    #[shared]
    struct Shared {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Get peripherals
        let dp: pac::Peripherals = cx.device;

        // Clock configuration. Use HSI at 16 MHz.
        let mut rcc = dp.RCC.freeze(Config::hsi16());

        rtt_init_print!();
        rprintln!("RTT init");

        // GPIO
        let gpiob = dp.GPIOB.split(&mut rcc);

        // Initialize the timer TIM6.
        //writeln!(
        //serial,
        //"Initialize monotonic timer (TIM6) at 7.8125 kHz (128 μs)"
        //)
        //.unwrap();
        let mono = Tim6Monotonic::initialize(dp.TIM6);

        let interval = INTERVAL_MS.millis();
        //writeln!(
        //serial,
        //"Schedule task every {} ms / {} ticks",
        //INTERVAL_MS,
        //interval.as_ticks()
        //)
        //.unwrap();

        // Spawn task "fizzbuzz"
        let _ = fizzbuzz::spawn();

        //writeln!(serial, "== Init done ==").unwrap();

        let local = Local {
            //serial,
            interval,
            counter: 1,
        };

        (Shared {}, local, init::Monotonics(mono))
    }

    #[task(local = [/*serial,*/ interval, counter])]
    fn fizzbuzz(cx: fizzbuzz::Context) {
        rprintln!("fizzbuzz!");
        // Access resources
        //let serial = cx.local.serial;
        let now = Instant::now().counts();
        let counter = cx.local.counter;

        // Fancy fizzbuzz implementation
        match (*counter % 3 == 0, *counter % 5 == 0) {
            (true, true) => rprintln!("fizzbuzz (now={:05})", now),
            (true, false) => rprintln!("    fizz (now={:05})", now),
            (false, true) => rprintln!("    buzz (now={:05})", now),
            _ => rprintln!("{:08} (now={:05})", *counter, now),
            //(true, true) => writeln!(serial, "fizzbuzz (now={:05})", now).unwrap(),
            //(true, false) => writeln!(serial, "    fizz (now={:05})", now).unwrap(),
            //(false, true) => writeln!(serial, "    buzz (now={:05})", now).unwrap(),
            //_ => writeln!(serial, "{:08} (now={:05})", *counter, now).unwrap(),
        }

        // Increment counter
        *counter += 1;

        // Re-schedule
        let _ = fizzbuzz::spawn_after(*cx.local.interval);
    }
}
