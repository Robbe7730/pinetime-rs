#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

mod drivers;
mod ui;

extern crate alloc;

#[rtic::app(device = nrf52832_hal::pac, dispatchers = [SWI0_EGU0])]
mod pinetimers {
    use crate::drivers::timer::MonoTimer;
    use crate::drivers::display::Display;
    use crate::drivers::touchpanel::TouchPanel;

    use crate::ui::screen::{Screen, ScreenDummy1};

    use rtt_target::{rprintln, rtt_init_print};

    use nrf52832_hal::pac::TIMER0;
    use nrf52832_hal::gpio::Level;
    use nrf52832_hal::gpiote::Gpiote;
    use nrf52832_hal::spim::{self, MODE_3, Spim};
    use nrf52832_hal::twim::{self, Twim};
    use nrf52832_hal::delay::Delay;

    use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
    use embedded_graphics_core::draw_target::DrawTarget;

    use fugit::ExtU32;

    use alloc::boxed::Box;

    #[monotonic(binds = TIMER0, default = true)]
    type Mono0 = MonoTimer<TIMER0>;

    type COLOR = Rgb565;

    #[shared]
    struct Shared {
        display: Display<COLOR>,
        gpiote: Gpiote,
        touchpanel: TouchPanel,

        current_screen: Box<dyn Screen<COLOR>>,

        counter: usize,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();
        rprintln!("Pijn tijd");

        // Set up heap
        unsafe {
            let heap_start = 0x2000_1000;
            let heap_end = 0x2001_0000;
            crate::HEAP.lock().init(heap_start, heap_end - heap_start);
        }

        let gpio = nrf52832_hal::gpio::p0::Parts::new(ctx.device.P0);

        let timer0: MonoTimer<TIMER0> = MonoTimer::new(ctx.device.TIMER0);

        // Set up GPIOTE
        let gpiote = Gpiote::new(ctx.device.GPIOTE);

        // Set up button
        gpio.p0_15.into_push_pull_output(Level::High);
        let button_input_pin = gpio.p0_13.into_floating_input().degrade();

        // Fire event on button press
        gpiote.channel0()
            .input_pin(&button_input_pin)
            .lo_to_hi()
            .enable_interrupt();

        // Set up SPI
        let spi_pins = spim::Pins {
            sck: gpio.p0_02.into_push_pull_output(Level::Low).degrade(),
            mosi: Some(gpio.p0_03.into_push_pull_output(Level::Low).degrade()),
            miso: Some(gpio.p0_04.into_floating_input().degrade())
        };
        let spi = Spim::new(
            ctx.device.SPIM0,
            spi_pins,
            spim::Frequency::M8,
            MODE_3,
            0
        );

        // Set up TWIM (I²C)
        let twim_pins = twim::Pins {
            sda: gpio.p0_06.into_floating_input().degrade(),
            scl: gpio.p0_07.into_floating_input().degrade(),
        };
        let mut twim = Twim::new(
            ctx.device.TWIM1,
            twim_pins,
            twim::Frequency::K250
        );
        twim.enable();

        // Set up touch panel
        let tp_int_pin = gpio.p0_28.into_floating_input().degrade();
        gpiote.channel1()
            .input_pin(&tp_int_pin)
            .lo_to_hi()
            .enable_interrupt();
        let touchpanel = TouchPanel::new(twim);

        // Set up display
        let display: Display<COLOR> = Display::new(
            // Backlight pins
            gpio.p0_14.into_push_pull_output(Level::High).degrade(),
            gpio.p0_22.into_push_pull_output(Level::High).degrade(),
            gpio.p0_23.into_push_pull_output(Level::High).degrade(),

            // Command/Data pin
            gpio.p0_18.into_push_pull_output(Level::Low).degrade(),

            // Chip Select pin
            gpio.p0_25.into_push_pull_output(Level::High).degrade(),

            // Reset pin
            gpio.p0_26.into_push_pull_output(Level::High).degrade(),

            spi,
            Delay::new(ctx.core.SYST),
        );

        // Set up the UI
        let screen: Box<dyn Screen<COLOR>> = Box::new(ScreenDummy1::new());

        display_init::spawn().unwrap();

        (Shared {
            display,
            gpiote,
            touchpanel,

            current_screen: screen,

            counter: 0,
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
            display.init();
            display.clear(COLOR::WHITE).unwrap();
        });
        draw_screen::spawn().unwrap();
    }

    #[task(binds = GPIOTE, shared = [gpiote, counter, touchpanel, current_screen])]
    fn gpiote_interrupt(ctx: gpiote_interrupt::Context) {
        (
            ctx.shared.gpiote,
            ctx.shared.counter,
            ctx.shared.touchpanel,
            ctx.shared.current_screen
        ).lock(|gpiote, counter, touchpanel, current_screen| {
            if gpiote.channel0().is_event_triggered() {
                *counter += 1;
            } else if gpiote.channel1().is_event_triggered() {
                touchpanel.handle_interrupt(Some(current_screen.get_event_handler()));
            } else {
                panic!("Unknown channel triggered");
            }
            gpiote.reset_events()
        });
        draw_screen::spawn().unwrap();
    }

    #[task(shared = [display, current_screen])]
    fn draw_screen(ctx: draw_screen::Context) {
        (ctx.shared.display, ctx.shared.current_screen).lock(|display, current_screen| {
            current_screen.draw(display);
        });
    }

    #[task(shared = [current_screen])]
    fn transition(mut ctx: transition::Context, new_screen: Box<dyn Screen<COLOR>>) {
        (ctx.shared.current_screen).lock(|current_screen| {
            *current_screen = new_screen;
        });
    }
}

use rtt_target::rprintln;

use core::panic::PanicInfo;

use linked_list_allocator::LockedHeap;

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
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
    panic!("Out of memory");
}
