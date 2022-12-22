#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use heapless::Vec;
use panic_rtt_target as _;
use rtic::app;
use rtt_target::{rprintln, rtt_init_print};
use stm32l4xx_hal::gpio::{gpiob::PB3, Output, PushPull};
use stm32l4xx_hal::prelude::*;
use systick_monotonic::{fugit::Duration, Systick};

#[app(device = stm32l4xx_hal::pac, dispatchers = [SPI3])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PB3<Output<PushPull>>,
        intervals: Vec<u32, 6>,
    }

    #[monotonic(binds = SysTick, default = true)]
    type MonoTimer = Systick<1000>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Setup clocks
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();
        let mut pwr = cx.device.PWR.constrain(&mut rcc.apb1r1);
        let mono = Systick::new(cx.core.SYST, 72_000_000);

        rtt_init_print!();
        rprintln!("init");

        let _clocks = rcc.cfgr.sysclk(72.MHz()).freeze(&mut flash.acr, &mut pwr);

        // Setup LED
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.ahb2);
        let mut led = gpiob
            .pb3
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
        led.set_low();

        // Simple heart beat LED on/off sequence
        let mut intervals: Vec<u32, 6> = Vec::new();
        intervals.push(30).unwrap(); // P Wave
        intervals.push(40).unwrap(); // PR Segment
        intervals.push(120).unwrap(); // QRS Complex
        intervals.push(30).unwrap(); // ST Segment
        intervals.push(60).unwrap(); // T Wave
        intervals.push(720).unwrap(); // Rest

        // Schedule the blinking task
        blink::spawn(0).unwrap();

        (Shared {}, Local { led, intervals }, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            core::hint::spin_loop();
        }
    }

    #[task(local = [led, intervals])]
    fn blink(cx: blink::Context, state: usize) {
        rprintln!("blink");
        let duration = cx.local.intervals[state];
        let next_state = (state + 1) % cx.local.intervals.len();

        cx.local.led.toggle();

        let _ = blink::spawn_after(
            Duration::<u64, 1, 1000>::from_ticks(duration as u64),
            next_state,
        );
    }
}
