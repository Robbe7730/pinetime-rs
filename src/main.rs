#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

mod timer;
mod display;
mod allocator;

extern crate alloc;

#[rtic::app(device = nrf52832_hal::pac, dispatchers = [SWI0_EGU0])]
mod pinetimers {
    use crate::timer::MonoTimer;
    use crate::display::Display;

    use rtt_target::{rprintln, rtt_init_print};

    use nrf52832_hal::pac::{TIMER0, SPIM1};
    use nrf52832_hal::gpio::Level;
    use nrf52832_hal::spim::{Frequency, MODE_3, Pins, Spim};
    use nrf52832_hal::delay::Delay;

    use fugit::ExtU32;

    #[monotonic(binds = TIMER0, default = true)]
    type Mono0 = MonoTimer<TIMER0>;

    #[shared]
    struct Shared {
        display: Display<Spim<SPIM1>>,
        i: u16,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();
        rprintln!("Pijn tijd");

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
            Delay::new(ctx.core.SYST),
        );

        display_init::spawn_after(1.secs()).unwrap();

        (Shared {
            display,
            i: 0,
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
            display.software_reset();
            display.set_sleep(false);
            display.set_display_on(true);
            display.set_brightness(0x8);
        });
        do_something::spawn_after(1.secs()).unwrap();
    }

    #[task(shared = [display, i])]
    fn do_something(ctx: do_something::Context) {
        (ctx.shared.display, ctx.shared.i).lock(|display, i| {
            display.set_brightness((*i % 8) as u8);
            *i += 1;
            rprintln!("{}", i);
        });
        do_something::spawn_after(1.secs()).unwrap();
    }
}

use rtt_target::rprintln;

use core::panic::PanicInfo;
use core::cell::UnsafeCell;

use allocator::BumpPointerAlloc;

use alloc::alloc::Layout;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("----- PANIC -----");
    rprintln!("{:#?}", info);
    loop {
        cortex_m::asm::bkpt();
    }
}

#[global_allocator]
static HEAP: BumpPointerAlloc = BumpPointerAlloc {
    head: UnsafeCell::new(0x2000_1000),
    end: 0x2001_0000,
};


#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
    panic!("Out of memory");
}
