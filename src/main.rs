#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

mod timer;
mod display;

extern crate alloc;

#[rtic::app(device = nrf52832_hal::pac, dispatchers = [SWI0_EGU0])]
mod pinetimers {
    use crate::timer::MonoTimer;
    use crate::display::Display;

    use rtt_target::{rprintln, rtt_init_print};

    use nrf52832_hal::pac::TIMER0;
    use nrf52832_hal::gpio::{Level, Pin, Input, Floating};
    use nrf52832_hal::spim::{Frequency, MODE_3, Pins, Spim};
    use nrf52832_hal::delay::Delay;
    use nrf52832_hal::prelude::InputPin;

    use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
    use embedded_graphics::prelude::{Point};
    use embedded_graphics::text::{Text, Baseline};
    use embedded_graphics::text::renderer::CharacterStyle;
    use embedded_graphics::mono_font::ascii::FONT_10X20;
    use embedded_graphics::mono_font::MonoTextStyle;
    use embedded_graphics::draw_target::DrawTarget;
    use embedded_graphics::Drawable;

    use fugit::ExtU32;

    use alloc::format;

    #[monotonic(binds = TIMER0, default = true)]
    type Mono0 = MonoTimer<TIMER0>;

    #[shared]
    struct Shared {
        display: Display<Rgb565>,
        button: Pin<Input<Floating>>,
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

        // Set up button
        gpio.p0_15.into_push_pull_output(Level::High);
        let button = gpio.p0_13.into_floating_input().degrade();

        // Set up SPI
        let spi_pins = Pins {
            sck: gpio.p0_02.into_push_pull_output(Level::Low).degrade(),
            mosi: Some(gpio.p0_03.into_push_pull_output(Level::Low).degrade()),
            miso: Some(gpio.p0_04.into_floating_input().degrade())
        };
        let spi = Spim::new(
            ctx.device.SPIM0,
            spi_pins,
            Frequency::M8,
            MODE_3,
            0
        );

        // Set up display
        let display: Display<Rgb565> = Display::new(
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

        display_init::spawn_after(1.secs()).unwrap();

        (Shared {
            display,
            counter: 0,
            button,
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
            display.clear(Rgb565::WHITE).unwrap();
        });
        write_counter::spawn().unwrap();

    }

    #[task(shared = [display, counter, button])]
    fn write_counter(ctx: write_counter::Context) {
        write_counter::spawn_after(1.secs()).unwrap();

        (ctx.shared.display, ctx.shared.counter, ctx.shared.button).lock(|display, counter, button| {
            let mut character_style = MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK);
            character_style.set_background_color(Some(Rgb565::WHITE));
            Text::with_baseline(&format!("{}", counter), Point::new(0, 0), character_style, Baseline::Top)
                .draw(display)
                .unwrap();

            let pressed_text = if button.is_high().unwrap() {
                "pressed    "
            } else {
                "not pressed"
            };

            Text::with_baseline(&format!("{}", pressed_text), Point::new(0, 20), character_style, Baseline::Top)
                .draw(display)
                .unwrap();

            *counter += 1;
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
