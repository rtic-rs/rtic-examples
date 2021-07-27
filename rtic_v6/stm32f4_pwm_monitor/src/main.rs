#![deny(unsafe_code)]
#![no_main]
#![no_std]
#![allow(unused_imports)]

use panic_rtt_target as _panic_handler;

/* declare a submodule for handling tim8 interrupts */
mod tim8;

/* declare the RTIC application itself */
#[rtic::app(device = stm32f4xx_hal::stm32, peripherals = true)]
mod app {

    /* bring dependencies into scope */
    use rtt_target::{rprintln, rtt_init_print};
    use stm32f4xx_hal::{
        gpio::{gpioc::PC6, Alternate},
        prelude::*,
        pwm_input::PwmInput,
        stm32::TIM8,
        timer::Timer,
    };
    /// PWM input monitor type
    pub(crate) type PwmMonitor = PwmInput<TIM8, PC6<Alternate<3>>>;

    /* resources shared across RTIC tasks */
    #[shared]
    struct Shared {
        /// the last observed position of the turret
        last_observed_turret_position: f32,
    }

    /* resources local to specific RTIC tasks */
    #[local]
    struct Local {
        monitor: PwmMonitor,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Enable RTT logging
        rtt_init_print!();
        rprintln!("hello, world!");
        // retrieve the RCC register, which is needed to obtain a handle to the clocks
        let rcc = ctx.device.RCC.constrain();
        // then retreive the clocks, so we can configure timers later on
        let clocks = rcc.cfgr.freeze();

        // obtain a reference to the GPIOC register block, so we can configure pins on the PC bus.
        let gpioc = ctx.device.GPIOC.split();

        // Configure one of TIM8's CH1 pins, so that its attached to the peripheral.
        // We need to do this since the pins are multiplexed across multiple peripherals
        let tim8_cc1 = gpioc.pc6.into_alternate();

        // Configure TIM8 into PWM input mode.
        // This requires a "best guess" of the input frequency in order to be accurate.
        // Note: as a side-effect TIM8's interrupt is enabled and fires whenever a capture-compare
        //      cycle is complete. See the reference manual's paragraphs on PWM Input.
        let monitor = Timer::new(ctx.device.TIM8, &clocks).pwm_input(240.hz(), tim8_cc1);

        // lastly return the shared and local resources, as per RTIC's spec.
        (
            Shared {
                last_observed_turret_position: 0.0,
            },
            Local { monitor },
            init::Monotonics(),
        )
    }

    /* bring tim8's interrupt handler into scope */
    use crate::tim8::tim8_cc;

    // RTIC docs specify we can modularize the code by using these `extern` blocks.
    // This allows us to specify the tasks in other modules and still work within
    // RTIC's infrastructure.
    extern "Rust" {
        #[task(binds=TIM8_CC, local=[monitor], shared=[last_observed_turret_position])]
        fn tim8_cc(context: tim8_cc::Context);
    }
}
