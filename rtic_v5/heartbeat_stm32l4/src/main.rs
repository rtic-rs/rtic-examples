#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;
use embedded_hal::digital::v2::OutputPin;
use rtic::app;
use rtic::cyccnt::U32Ext;

use stm32l4xx_hal::gpio::{gpiob::PB3, Output, PushPull, State};
use stm32l4xx_hal::prelude::*;

const PERIOD: u32 = 100_000_000;

// We need to pass monotonic = rtic::cyccnt::CYCCNT to use schedule feature fo RTIC
#[app(device = stm32l4xx_hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    // Global resources (global variables) are defined here and initialized with the 
    // `LateResources` struct in init
    struct Resources {
        led: PB3<Output<PushPull>>,
    }

    #[init(schedule = [blinker])]
    fn init(cx: init::Context) -> init::LateResources {
        // Enable cycle counter
        let mut core = cx.core;
        core.DWT.enable_cycle_counter();

        // Setup clocks
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();
        let _clocks = rcc
            .cfgr
            .sysclk(72.mhz())
            .freeze(&mut flash.acr);

        // Setup LED
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.ahb2);
        let led = gpiob
            .pb3
            .into_push_pull_output_with_state(&mut gpiob.moder, &mut gpiob.otyper, State::Low);

        // Schedule the blinking task
        cx.schedule.blinker(cx.start + PERIOD.cycles()).unwrap();

        init::LateResources { led: led }
    }

    #[task(resources = [led], schedule = [blinker])]
    fn blinker(cx: blinker::Context) {
        // Use the safe local `static mut` of RTIC
        static mut LED_STATE: bool = false;

        if *LED_STATE {
            cx.resources.led.set_high().unwrap();
            *LED_STATE = false;
        } else {
            cx.resources.led.set_low().unwrap();
            *LED_STATE = true;
        }
        cx.schedule.blinker(cx.scheduled + PERIOD.cycles()).unwrap();
    }

    extern "C" {
        fn EXTI0();
    }
};
