//! Demonstrates the use of an independent watchdog (IWDG).
//! The watchdog is activated and will reset the board if it isn't fed every 500ms.
//! Pressing the button connected to PC13 will disable the watchdog feeding which will trigger
//! a hardware reset.

#![deny(warnings)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_probe as _;

use defmt_rtt as _;

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [EXTI1])]
mod app {
    use stm32f4xx_hal::{
        gpio::{Edge, Input, PC13},
        pac,
        prelude::*,
        timer::MonoTimerUs,
        watchdog::IndependentWatchdog,
    };

    #[monotonic(binds = TIM2, default = true)]
    type MicrosecMono = MonoTimerUs<pac::TIM2>;

    #[shared]
    struct Shared {
        do_feed_watchdog: bool,
    }

    #[local]
    struct Local {
        button: PC13<Input>,
        watchdog: IndependentWatchdog,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut syscfg = ctx.device.SYSCFG.constrain();

        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        let mono = ctx.device.TIM2.monotonic_us(&clocks);

        let gpioc = ctx.device.GPIOC.split();

        let mut button = gpioc.pc13.into_pull_down_input();
        button.make_interrupt_source(&mut syscfg);
        button.enable_interrupt(&mut ctx.device.EXTI);
        button.trigger_on_edge(&mut ctx.device.EXTI, Edge::Falling);

        let mut watchdog = IndependentWatchdog::new(ctx.device.IWDG);
        watchdog.start(500u32.millis());
        feed_watchdog::spawn().ok();

        defmt::info!("init done!");

        (
            Shared {
                do_feed_watchdog: true,
            },
            Local { button, watchdog },
            init::Monotonics(mono),
        )
    }

    /// React on the button click and disable the watchdog. This will trigger a hardware reset.
    // see here for why this is EXTI15_10: https://github.com/stm32-rs/stm32f4xx-hal/blob/6d0c29233a4cd1f780b2fef3e47ef091ead6cf4a/src/gpio/exti.rs#L8-L23
    #[task(binds = EXTI15_10, local = [button], shared = [do_feed_watchdog])]
    fn button_click(mut ctx: button_click::Context) {
        ctx.local.button.clear_interrupt_pending_bit();
        defmt::info!("button pressed => not feeding the watchdog anymore.");
        ctx.shared.do_feed_watchdog.lock(|do_feed_watchdog| {
            *do_feed_watchdog = false;
        });
    }

    /// Feed the watchdog to avoid hardware reset.
    #[task(priority=1, local=[watchdog], shared=[do_feed_watchdog])]
    fn feed_watchdog(mut ctx: feed_watchdog::Context) {
        defmt::trace!("feeding the watchdog!");
        ctx.local.watchdog.feed();
        ctx.shared.do_feed_watchdog.lock(|do_feed_watchdog| {
            if *do_feed_watchdog {
                feed_watchdog::spawn_after(100.millis()).ok();
            }
        });
    }
}
