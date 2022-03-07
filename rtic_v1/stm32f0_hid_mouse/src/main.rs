/*
#![deny(unsafe_code)]
*/
#![deny(warnings)]
#![no_std]
#![no_main]

use cortex_m::interrupt::free as disable_interrupts;
use panic_halt as _;
use rtic::app;
use rtic::Exclusive;
use rtic::Mutex;
use rtt_target::{rprintln, rtt_init_print};
use stm32f0xx_hal::{
    gpio::gpioa::{PA0, PA1, PA15, PA2, PA3, PA4, PA5, PA6, PA7},
    gpio::gpiob::{PB1, PB3, PB4},
    gpio::{Input, Output, PullUp, PushPull},
    pac,
    prelude::*,
    usb,
};
use usb_device::{bus::UsbBusAllocator, prelude::*};
use usbd_hid::{
    descriptor::{MouseReport, SerializedDescriptor},
    hid_class::HIDClass,
};

#[app(device = stm32f0xx_hal::pac, peripherals = true)]
mod app {
    use super::*;

    #[local]
    struct Local {
        usr_led: PB1<Output<PushPull>>,
        usb_device: UsbDevice<'static, usb::UsbBusType>,
    }

    #[shared]
    struct Shared {
        usb_hid: HIDClass<'static, usb::UsbBusType>,
        exti: pac::EXTI,
        _button3: PA15<Input<PullUp>>,
        _button4: PB4<Input<PullUp>>,
        _button5: PB3<Input<PullUp>>,
        _tb_left: PA4<Input<PullUp>>,
        _tb_up: PA5<Input<PullUp>>,
        _tb_right: PA6<Input<PullUp>>,
        _tb_down: PA7<Input<PullUp>>,
        bbled_red: PA0<Output<PushPull>>,
        bbled_grn: PA1<Output<PushPull>>,
        bbled_blu: PA2<Output<PushPull>>,
        bbled_wht: PA3<Output<PushPull>>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // RTT handler
        rtt_init_print!();

        // Alias peripherals
        let mut dp: pac::Peripherals = ctx.device;

        // This enables clock for SYSCFG and remaps USB pins to PA9 and PA10.
        usb::remap_pins(&mut dp.RCC, &mut dp.SYSCFG);

        rprintln!("Initializing peripherals");
        let mut rcc = dp
            .RCC
            .configure()
            .usbsrc(stm32f0xx_hal::rcc::USBClockSource::HSI48)
            .hsi48()
            .enable_crs(dp.CRS)
            .sysclk(48.mhz())
            .pclk(24.mhz())
            .freeze(&mut dp.FLASH);

        // Set up GPIO registers for USR LED and Buttons
        let gpiob = dp.GPIOB.split(&mut rcc);
        let (usr_led, _button4, _button5) = disable_interrupts(|cs| {
            (
                gpiob.pb1.into_push_pull_output(cs),
                gpiob.pb4.into_pull_up_input(cs),
                gpiob.pb3.into_pull_up_input(cs),
            )
        });

        // LEDs and USB
        let gpioa = dp.GPIOA.split(&mut rcc);
        let (
            bbled_red,
            bbled_grn,
            bbled_blu,
            bbled_wht,
            _tb_left,
            _tb_up,
            _tb_right,
            _tb_down,
            _button3,
            usb_dm,
            usb_dp,
        ) = disable_interrupts(|cs| {
            (
                gpioa.pa0.into_push_pull_output(cs),
                gpioa.pa1.into_push_pull_output(cs),
                gpioa.pa2.into_push_pull_output(cs),
                gpioa.pa3.into_push_pull_output(cs),
                gpioa.pa4.into_pull_up_input(cs),
                gpioa.pa5.into_pull_up_input(cs),
                gpioa.pa6.into_pull_up_input(cs),
                gpioa.pa7.into_pull_up_input(cs),
                gpioa.pa15.into_pull_up_input(cs),
                gpioa.pa11,
                gpioa.pa12,
            )
        });

        // Power on bbled dance
        //bbled_red.toggle().ok();

        // Enable external interrupt for 3 aux buttons...
        dp.SYSCFG.exticr1.write(|w| w.exti3().pb3());
        // dp.SYSCFG.exticr2.write(|w| { w.exti4().pb4() }); // Disable spare button in favor of tb_left
        dp.SYSCFG.exticr4.write(|w| w.exti15().pa15());
        //... and for pulses on trackball
        dp.SYSCFG.exticr2.write(|w| w.exti4().pa4());
        dp.SYSCFG.exticr2.write(|w| w.exti5().pa5());
        dp.SYSCFG.exticr2.write(|w| w.exti6().pa6());
        dp.SYSCFG.exticr2.write(|w| w.exti7().pa7());

        // Set interrupt mask for all the above
        dp.EXTI.imr.write(|w| {
            w.mr3().set_bit();
            w.mr4().set_bit();
            w.mr5().set_bit();
            w.mr6().set_bit();
            w.mr7().set_bit();
            w.mr15().set_bit()
        });

        // Set interrupt rising trigger
        dp.EXTI.rtsr.write(|w| {
            w.tr3().set_bit();
            w.tr4().set_bit();
            w.tr5().set_bit();
            w.tr6().set_bit();
            w.tr7().set_bit();
            w.tr15().set_bit()
        });

        let usb = usb::Peripheral {
            usb: dp.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };

        rprintln!("Preparing HID mouse...");
        static mut USB_BUS: Option<UsbBusAllocator<usb::UsbBusType>> = None;
        unsafe {
            USB_BUS = Some(usb::UsbBus::new(usb));
        };
        let usb_hid = HIDClass::new(
            unsafe { USB_BUS.as_ref().unwrap() },
            MouseReport::desc(),
            60,
        );

        rprintln!("Defining USB parameters...");
        let usb_device =
            UsbDeviceBuilder::new(unsafe { USB_BUS.as_ref().unwrap() }, UsbVidPid(0, 0x3821))
                .manufacturer("JoshFTW")
                .product("BBTrackball")
                .serial_number("RustFW")
                .device_class(0x00)
                .device_sub_class(0x00)
                .device_protocol(0x00)
                .build();

        rprintln!("Instantiating dp.EXTI...");
        let exti = dp.EXTI;

        rprintln!("Defining shared resources...");
        let shared = Shared {
            usb_hid,
            exti,
            _button3,
            _button4,
            _button5,
            _tb_left,
            _tb_up,
            _tb_right,
            _tb_down,
            bbled_red,
            bbled_grn,
            bbled_blu,
            bbled_wht,
        };

        (
            shared,
            Local {
                usr_led,
                usb_device,
            },
            init::Monotonics(),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
            cortex_m::asm::wfi();
        }
    }

    #[task(binds = EXTI2_3, shared = [exti])]
    fn exti2_3_interrupt(mut ctx: exti2_3_interrupt::Context) {
        rprintln!("Interrupts happening on EXTI2_3");

        match ctx.shared.exti.lock(|exti| exti.pr.read().bits()) {
            0x8 => {
                rprintln!("PB3 triggered");
                ctx.shared
                    .exti
                    .lock(|exti| exti.pr.write(|w| w.pif3().set_bit())); // Clear interrupt
            }

            _ => rprintln!("Some other bits were pushed around on EXTI2_3 ;)"),
        }
    }

    #[task(binds = EXTI4_15, local = [usr_led], shared = [exti, usb_hid, bbled_red, bbled_grn, bbled_wht, bbled_blu])]
    fn exti_4_15_interrupt(mut ctx: exti_4_15_interrupt::Context) {
        rprintln!("Interrupts happening on EXTI for PA15...");

        match ctx.shared.exti.lock(|exti| exti.pr.read().bits()) {
            0x8000 => {
                rprintln!("PA15 triggered");
                ctx.shared
                    .exti
                    .lock(|exti| exti.pr.write(|w| w.pif15().set_bit())); // Clear interrupt

                ctx.shared
                    .usb_hid
                    .lock(|hid| super::send_mouse_report(Exclusive(hid), 0, 0, 1));
                ctx.local.usr_led.toggle().ok();
            }
            0x10 => {
                rprintln!("tb_left triggered!");
                ctx.shared
                    .exti
                    .lock(|exti| exti.pr.write(|w| w.pif4().set_bit()));
                ctx.shared
                    .usb_hid
                    .lock(|hid| super::send_mouse_report(Exclusive(hid), 5, 0, 0));
                ctx.local.usr_led.toggle().ok();
            }
            0x20 => {
                rprintln!("tb_up triggered!");
                ctx.shared
                    .exti
                    .lock(|exti| exti.pr.write(|w| w.pif5().set_bit()));

                ctx.shared
                    .usb_hid
                    .lock(|hid| super::send_mouse_report(Exclusive(hid), 0, 5, 0));
                ctx.local.usr_led.toggle().ok();
            }
            0x40 => {
                rprintln!("tb_right triggered!");
                ctx.shared
                    .exti
                    .lock(|exti| exti.pr.write(|w| w.pif6().set_bit()));

                ctx.shared
                    .usb_hid
                    .lock(|hid| super::send_mouse_report(Exclusive(hid), -5, 0, 0));
                ctx.local.usr_led.toggle().ok();
            }
            0x80 => {
                rprintln!("tb_down triggered!");
                ctx.shared
                    .exti
                    .lock(|exti| exti.pr.write(|w| w.pif7().set_bit()));

                ctx.shared
                    .usb_hid
                    .lock(|hid| super::send_mouse_report(Exclusive(hid), 0, -5, 0));
                ctx.local.usr_led.toggle().ok();
            }

            _ => rprintln!("Some other bits were pushed around on EXTI4_15 ;)"),
        }
    }

    #[task(binds = USB, local = [usb_device], shared = [usb_hid])]
    fn usb_handler(mut ctx: usb_handler::Context) {
        rprintln!("USB interrupt received.");

        let device = ctx.local.usb_device;
        ctx.shared.usb_hid.lock(|hid| {
            // USB dev poll only in the interrupt handler
            device.poll(&mut [hid]);
        });
    }
}

fn send_mouse_report(
    mut shared_hid: impl Mutex<T = HIDClass<'static, usb::UsbBusType>>,
    x: i8,
    y: i8,
    buttons: u8,
) {
    let mr = MouseReport {
        x,
        y,
        buttons,
        wheel: 0,
        pan: 0,
    };

    shared_hid.lock(|hid| {
        rprintln!("Sending mouse report...");
        hid.push_input(&mr).ok();
    });
}
