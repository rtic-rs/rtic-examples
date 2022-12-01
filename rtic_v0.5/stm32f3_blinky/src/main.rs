#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;
use embedded_hal::digital::v2::OutputPin;
use rtic::app;
use rtic::cyccnt::U32Ext;
use stm32f3xx_hal::gpio::{gpioe::PE10, Output, PushPull};
use stm32f3xx_hal::prelude::*;

const PERIOD: u32 = 10_000_000;

// We need to pass monotonic = rtic::cyccnt::CYCCNT to use schedule feature fo RTIC
#[app(device = stm32f3xx_hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    // Global resources (global variables) are defined here and initialized with the
    // `LateResources` struct in init
    struct Resources {
        led: PE10<Output<PushPull>>,
    }

    #[init(schedule = [blinker])]
    fn init(cx: init::Context) -> init::LateResources {
        // Enable cycle counter
        let mut core = cx.core;
        core.DWT.enable_cycle_counter();

        let device: stm32f3xx_hal::pac::Peripherals = cx.device;

        // Setup clocks
        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();
        let _clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(72.MHz())
            .pclk1(36.MHz())
            .freeze(&mut flash.acr);

        // Setup LED
        let mut gpioe = device.GPIOE.split(&mut rcc.ahb);
        let mut led = gpioe
            .pe10
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
        led.set_high().unwrap();

        // Schedule the blinking task
        cx.schedule.blinker(cx.start + PERIOD.cycles()).unwrap();

        init::LateResources { led }
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
