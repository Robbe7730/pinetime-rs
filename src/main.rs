#![no_main]
#![no_std]

mod timer;
mod display;

use rtt_target::rprintln;

use core::panic::PanicInfo;

#[rtic::app(device = nrf52832_hal::pac, dispatchers = [SWI0_EGU0])]
mod pinetimers {
    use crate::timer::MonoTimer;
    use crate::display::Display;

    use rtt_target::{rprintln, rtt_init_print};

    use nrf52832_hal::pac::TIMER0;
    use nrf52832_hal::gpio::Level;
    use nrf52832_hal::spim::{Frequency, MODE_3, Pins, Spim};

    use fugit::ExtU32;

    #[monotonic(binds = TIMER0, default = true)]
    type Mono0 = MonoTimer<TIMER0>;

    #[shared]
    struct Shared {
        display: Display,
        inverted: bool,
        brightness: u8,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();
        rprintln!("Pijn tijd");

        display_init::spawn().unwrap();

        let gpio = nrf52832_hal::gpio::p0::Parts::new(ctx.device.P0);

        let timer0: MonoTimer<TIMER0> = MonoTimer::new(ctx.device.TIMER0);

        // Set up display
        let display_pins = Pins {
            sck: gpio.p0_02.into_push_pull_output(Level::Low).degrade(),
            mosi: Some(gpio.p0_03.into_push_pull_output(Level::Low).degrade()),
            miso: Some(gpio.p0_04.into_floating_input().degrade())
        };
        let display_spi = Spim::new(
            ctx.device.SPIM1,
            display_pins,
            Frequency::M8,
            MODE_3,
            0
        );
        let display = Display::new(
            gpio.p0_14.into_push_pull_output(Level::High).degrade(),
            gpio.p0_22.into_push_pull_output(Level::High).degrade(),
            gpio.p0_23.into_push_pull_output(Level::High).degrade(),
            gpio.p0_25.into_push_pull_output(Level::High).degrade(),
            gpio.p0_18.into_push_pull_output(Level::High).degrade(),
            display_spi,
        );

        (Shared {
            display,
            inverted: false,
            brightness: 0
        }, Local {}, init::Monotonics(timer0))
    }
    
    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            rprintln!("IDLE");
            cortex_m::asm::wfi();
        }
    }

    #[task(shared = [display])]
    fn display_init(mut ctx: display_init::Context) {
        ctx.shared.display.lock(|display| {
            display.set_sleep(false);
            display.set_display_on(true);
            display.set_brightness(0x8);
        });
        toggle_invert::spawn_after(1_u32.secs()).unwrap();
    }

    #[task(shared = [display, inverted, brightness])]
    fn toggle_invert(ctx: toggle_invert::Context) {
        (ctx.shared.display, ctx.shared.inverted, ctx.shared.brightness).lock(|display, inverted, brightness| {
            display.set_brightness(*brightness);

            *brightness = *brightness + 1;
        });
        toggle_invert::spawn_after(1_u32.secs()).unwrap();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("----- PANIC -----");
    rprintln!("{:#?}", info);
    loop {
        cortex_m::asm::bkpt();
    }
}
