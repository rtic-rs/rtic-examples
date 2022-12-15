#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use heapless::Vec;

use rtic::app;
use rtic::cyccnt::U32Ext;

use stm32l4xx_hal::gpio::{gpiob::PB3, Output, PushPull, State};
use stm32l4xx_hal::prelude::*;

const BEATS_PER_MIN: u32 = 60;
const CLK_SPEED_MHZ: u32 = 72;

// Cycles per thousandth of beat
const MILLI_BEAT: u32 = CLK_SPEED_MHZ * 60_000 / BEATS_PER_MIN;

// We need to pass monotonic = rtic::cyccnt::CYCCNT to use schedule feature fo RTIC
#[app(device = stm32l4xx_hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    // Global resources (global variables) are defined here and initialized with the
    // `LateResources` struct in init
    struct Resources {
        led: PB3<Output<PushPull>>,
        intervals: Vec<u32, 6>,
    }

    #[init(schedule = [blinker])]
    fn init(cx: init::Context) -> init::LateResources {
        // Enable cycle counter
        let mut core = cx.core;
        core.DWT.enable_cycle_counter();

        // Setup clocks
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(CLK_SPEED_MHZ.mhz()).freeze(&mut flash.acr);

        // Setup LED
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.ahb2);
        let led = gpiob.pb3.into_push_pull_output_with_state(
            &mut gpiob.moder,
            &mut gpiob.otyper,
            State::Low,
        );

        // Simple heart beat LED on/off sequence
        let mut intervals: Vec<u32, 6> = Vec::new();
        intervals.push(MILLI_BEAT * 30).unwrap(); // P Wave
        intervals.push(MILLI_BEAT * 40).unwrap(); // PR Segment
        intervals.push(MILLI_BEAT * 120).unwrap(); // QRS Complex
        intervals.push(MILLI_BEAT * 30).unwrap(); // ST Segment
        intervals.push(MILLI_BEAT * 60).unwrap(); // T Wave
        intervals.push(MILLI_BEAT * 720).unwrap(); // Rest

        // Schedule the blinking task
        cx.schedule.blinker(cx.start, 0).unwrap();

        init::LateResources { led, intervals }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            core::hint::spin_loop();
        }
    }

    #[task(schedule = [blinker], resources = [led, &intervals])]
    fn blinker(cx: blinker::Context, state: usize) {
        let led = cx.resources.led;
        let intervals = cx.resources.intervals;
        let duration = intervals[state].cycles();
        let next_state = (state + 1) % intervals.len();

        if state % 2 == 0 {
            led.set_high().unwrap();
        } else {
            led.set_low().unwrap();
        }

        cx.schedule
            .blinker(cx.scheduled + duration, next_state)
            .unwrap();
    }

    extern "C" {
        fn EXTI0();
    }
};
