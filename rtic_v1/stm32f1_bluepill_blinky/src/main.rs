#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtic::app;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::gpio::PinState;
use stm32f1xx_hal::gpio::{gpioc::PC13, Output, PushPull};
use stm32f1xx_hal::prelude::*;
use systick_monotonic::{fugit::Duration, Systick};

#[app(device = stm32f1xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>,
        state: bool,
    }

    #[monotonic(binds = SysTick, default = true)]
    type MonoTimer = Systick<1000>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Setup clocks
        let mut flash = cx.device.FLASH.constrain();
        let rcc = cx.device.RCC.constrain();

        let mono = Systick::new(cx.core.SYST, 36_000_000);

        rtt_init_print!();
        rprintln!("init");

        let _clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(36.mhz())
            .pclk1(36.mhz())
            .freeze(&mut flash.acr);

        // Setup LED
        let mut gpioc = cx.device.GPIOC.split();
        let led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, PinState::Low);

        // Schedule the blinking task
        blink::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();

        (
            Shared {},
            Local { led, state: false },
            init::Monotonics(mono),
        )
    }

    #[task(local = [led, state])]
    fn blink(cx: blink::Context) {
        rprintln!("blink");
        if *cx.local.state {
            cx.local.led.set_high();
            *cx.local.state = false;
        } else {
            cx.local.led.set_low();
            *cx.local.state = true;
        }
        blink::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();
    }
}
