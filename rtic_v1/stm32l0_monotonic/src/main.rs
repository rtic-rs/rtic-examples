#![no_main]
#![no_std]

use core::fmt::Write;

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use rtic::app;
use stm32l0xx_hal::{pac, prelude::*, rcc::Config, serial};
use systick_monotonic::{
    fugit::{ExtU32, MillisDurationU64},
    Systick,
};

const INTERVAL_MS: u32 = 500;

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
    #[monotonic(binds = SysTick, default = true)]
    type Tonic = Systick<1000>;

    #[local]
    struct Local {
        /// Serial debug output
        serial: serial::Serial<pac::USART2>,

        /// Timer interval
        interval: MillisDurationU64,

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

        // Initialize serial port(s)
        let mut serial = serial::Serial::usart2(
            dp.USART2,
            gpiob.pb6.into_floating_input(),
            gpiob.pb7.into_floating_input(),
            serial::Config {
                baudrate: 57_600.Bd(),
                wordlength: serial::WordLength::DataBits8,
                parity: serial::Parity::ParityNone,
                stopbits: serial::StopBits::STOP1,
            },
            &mut rcc,
        )
        .unwrap();

        // Initialize the timer
        writeln!(serial, "Initialize monotonic timer using SysTick at 1kHz").unwrap();

        let mono = Systick::new(cx.core.SYST, 16_000_000);

        let interval: MillisDurationU64 = INTERVAL_MS.millis().into();

        writeln!(
            serial,
            "Schedule task every {} ms / {} ticks",
            interval,
            interval.ticks(),
        )
        .unwrap();

        // Spawn task "fizzbuzz"
        let _ = fizzbuzz::spawn();

        writeln!(serial, "== Init done ==").unwrap();

        let local = Local {
            serial,
            interval,
            counter: 1,
        };

        (Shared {}, local, init::Monotonics(mono))
    }

    #[task(local = [serial, interval, counter])]
    fn fizzbuzz(cx: fizzbuzz::Context) {
        rprintln!("fizzbuzz!");
        // Access resources
        let serial = cx.local.serial;
        let now = monotonics::now();
        let counter = cx.local.counter;
        let interval = cx.local.interval;

        // Fancy fizzbuzz implementation
        match (*counter % 3 == 0, *counter % 5 == 0) {
            (true, true) => {
                rprintln!("fizzbuzz (now={:05})", now);
                writeln!(serial, "fizzbuzz (now={:05})", now).unwrap();
            }
            (true, false) => {
                rprintln!("    fizz (now={:05})", now);
                writeln!(serial, "    fizz (now={:05})", now).unwrap();
            }
            (false, true) => {
                rprintln!("    buzz (now={:05})", now);
                writeln!(serial, "    buzz (now={:05})", now).unwrap();
            }
            _ => {
                rprintln!("{:08} (now={:05})", *counter, now);
                writeln!(serial, "{:08} (now={:05})", *counter, now).unwrap();
            }
        }

        // Increment counter
        *counter += 1;

        // Re-schedule
        let _ = fizzbuzz::spawn_after(*interval);
    }
}
