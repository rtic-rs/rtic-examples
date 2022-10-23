#![no_std]
#![no_main]

#[rtic::app(
    device = rp_pico::hal::pac,
    dispatchers = [TIMER_IRQ_1]
)]
mod app {
    use rp2040_monotonic::{
        fugit::Duration,
        fugit::RateExtU32, // For .kHz() conversion funcs
        Rp2040Monotonic,
    };
    use rp_pico::hal::{
        clocks, gpio,
        gpio::pin::bank0::{Gpio2, Gpio25, Gpio3},
        gpio::pin::PushPullOutput,
        pac,
        sio::Sio,
        watchdog::Watchdog,
        I2C,
    };
    use rp_pico::XOSC_CRYSTAL_FREQ;

    use core::mem::MaybeUninit;
    use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};

    use panic_probe as _;

    const MONO_NUM: u32 = 1;
    const MONO_DENOM: u32 = 1000000;
    const ONE_SEC_TICKS: u64 = 1000000;

    type I2CBus = I2C<
        pac::I2C1,
        (
            gpio::Pin<Gpio2, gpio::FunctionI2C>,
            gpio::Pin<Gpio3, gpio::FunctionI2C>,
        ),
    >;

    #[monotonic(binds = TIMER_IRQ_0, default = true)]
    type Rp2040Mono = Rp2040Monotonic;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: gpio::Pin<Gpio25, PushPullOutput>,
        i2c: &'static mut I2CBus,
    }

    #[init(local=[
        // Task local initialized resources are static
        // Here we use MaybeUninit to allow for initialization in init()
        // This enables its usage in driver initialization
        i2c_ctx: MaybeUninit<I2CBus> = MaybeUninit::uninit()
    ])]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Configure the clocks, watchdog - The default is to generate a 125 MHz system clock
        let mut watchdog = Watchdog::new(ctx.device.WATCHDOG);
        let clocks = clocks::init_clocks_and_plls(
            XOSC_CRYSTAL_FREQ,
            ctx.device.XOSC,
            ctx.device.CLOCKS,
            ctx.device.PLL_SYS,
            ctx.device.PLL_USB,
            &mut ctx.device.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        // Init LED pin
        let sio = Sio::new(ctx.device.SIO);
        let gpioa = rp_pico::Pins::new(
            ctx.device.IO_BANK0,
            ctx.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut ctx.device.RESETS,
        );
        let mut led = gpioa.led.into_push_pull_output();
        led.set_low().unwrap();

        // Init I2C pins
        let sda_pin = gpioa.gpio2.into_mode::<gpio::FunctionI2C>();
        let scl_pin = gpioa.gpio3.into_mode::<gpio::FunctionI2C>();

        // Init I2C itself, using MaybeUninit to overwrite the previously
        // uninitialized i2c_ctx variable without dropping its value
        // (i2c_ctx definined in init local resources above)
        let i2c_tmp: &'static mut _ = ctx.local.i2c_ctx.write(I2C::i2c1(
            ctx.device.I2C1,
            sda_pin,
            scl_pin,
            100.kHz(),
            &mut ctx.device.RESETS,
            &clocks.system_clock,
        ));

        let mono = Rp2040Mono::new(ctx.device.TIMER);

        // Spawn heartbeat task
        heartbeat::spawn().unwrap();

        // Return resources and timer
        (
            Shared {},
            Local {
                led: led,
                i2c: i2c_tmp,
            },
            init::Monotonics(mono),
        )
    }

    #[task(local = [i2c, led])]
    fn heartbeat(ctx: heartbeat::Context) {
        // Flicker the built-in LED
        _ = ctx.local.led.toggle();

        // Congrats, you can use your i2c and have access to it here,
        // now to do something with it!

        // Re-spawn this task after 1 second
        let one_second = Duration::<u64, MONO_NUM, MONO_DENOM>::from_ticks(ONE_SEC_TICKS);
        heartbeat::spawn_after(one_second).unwrap();
    }
}
