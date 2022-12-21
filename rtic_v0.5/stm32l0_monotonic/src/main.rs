#![no_main]
#![no_std]

mod monotonic_stm32l0;

use core::fmt::Write;

use panic_halt as _;
use rtic::app;
use stm32l0xx_hal::{pac, prelude::*, rcc::Config, serial};

use crate::monotonic_stm32l0::{Duration, Instant, Tim6Monotonic, U16Ext};

const INTERVAL_MS: u16 = 500;

#[app(
    device = stm32l0xx_hal::pac,
    peripherals = true,
    monotonic = crate::monotonic_stm32l0::Tim6Monotonic,
)]
const APP: () = {
    struct Resources {
        /// Serial debug output
        debug: serial::Serial<pac::USART1>,

        /// Timer interval
        interval: Duration,
    }

    #[init(spawn = [fizzbuzz])]
    fn init(cx: init::Context) -> init::LateResources {
        // Get peripherals
        let dp: pac::Peripherals = cx.device;

        // Clock configuration. Use HSI at 16 MHz.
        let mut rcc = dp.RCC.freeze(Config::hsi16());

        // GPIO
        let gpiob = dp.GPIOB.split(&mut rcc);

        // Initialize serial port(s)
        let mut debug = serial::Serial::usart1(
            dp.USART1,
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

        // Initialize the timer TIM6.
        writeln!(
            debug,
            "Initialize monotonic timer (TIM6) at 7.8125 kHz (128 μs)"
        )
        .unwrap();
        Tim6Monotonic::initialize(dp.TIM6);

        // Spawn task "fizzbuzz"
        let interval = INTERVAL_MS.millis();
        writeln!(
            debug,
            "Schedule task every {} ms / {} ticks",
            INTERVAL_MS,
            interval.as_ticks()
        )
        .unwrap();
        cx.spawn.fizzbuzz().unwrap();

        writeln!(debug, "== Init done ==").unwrap();

        init::LateResources { debug, interval }
    }

    #[task(schedule = [fizzbuzz], resources = [debug, interval])]
    fn fizzbuzz(cx: fizzbuzz::Context) {
        static mut COUNTER: usize = 1;

        // Access resources
        let debug = cx.resources.debug;
        let scheduled = cx.scheduled.counts();
        let now = Instant::now().counts();

        // Classic fizzbuzz implementation
        if *COUNTER % 15 == 0 {
            writeln!(debug, "fizzbuzz (sched={:05}, now={:05})", scheduled, now).unwrap();
        } else if *COUNTER % 3 == 0 {
            writeln!(debug, "    fizz (sched={:05}, now={:05})", scheduled, now).unwrap();
        } else if *COUNTER % 5 == 0 {
            writeln!(debug, "    buzz (sched={:05}, now={:05})", scheduled, now).unwrap();
        } else {
            writeln!(
                debug,
                "{:08} (sched={:05}, now={:05})",
                *COUNTER, scheduled, now
            )
            .unwrap();
        }

        // Increment counter
        *COUNTER += 1;

        // Re-schedule
        cx.schedule
            .fizzbuzz(cx.scheduled + *cx.resources.interval)
            .unwrap();
    }

    extern "C" {
        fn SPI1();
    }
};
