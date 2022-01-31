#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

mod timer;
mod display;
mod touchpanel;

extern crate alloc;

#[rtic::app(device = nrf52832_hal::pac, dispatchers = [SWI0_EGU0])]
mod pinetimers {
    use crate::timer::MonoTimer;
    use crate::display::Display;
    use crate::touchpanel::{TouchPanel, TouchPanelEventHandler, TouchPoint};

    use rtt_target::{rprintln, rtt_init_print};

    use nrf52832_hal::pac::TIMER0;
    use nrf52832_hal::gpio::Level;
    use nrf52832_hal::gpiote::Gpiote;
    use nrf52832_hal::spim::{self, MODE_3, Spim};
    use nrf52832_hal::twim::{self, Twim};
    use nrf52832_hal::delay::Delay;

    use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
    use embedded_graphics::prelude::{Point, Size};
    use embedded_graphics::primitives::{Primitive, PrimitiveStyle, Rectangle};
    use embedded_graphics::text::{Text, Baseline};
    use embedded_graphics::text::renderer::CharacterStyle;
    use embedded_graphics::mono_font::MonoTextStyle;
    use embedded_graphics::mono_font::ascii::FONT_10X20;
    use embedded_graphics::draw_target::DrawTarget;
    use embedded_graphics::Drawable;

    use alloc::format;

    #[monotonic(binds = TIMER0, default = true)]
    type Mono0 = MonoTimer<TIMER0>;

    #[shared]
    struct Shared {
        display: Display<Rgb565>,
        gpiote: Gpiote,
        touchpanel: TouchPanel<MainTouchPanelHandler>,
        counter: usize,
    }

    #[local]
    struct Local {}

    pub struct MainTouchPanelHandler {}
    impl TouchPanelEventHandler for MainTouchPanelHandler {
        fn on_click(&self, touchpoint: TouchPoint) {
            draw_rectangle::spawn(touchpoint.x.into(), touchpoint.y.into(), 10, 10, Rgb565::RED);
        }

        fn on_slide(&self, touchpoint: TouchPoint) {
            draw_rectangle::spawn(touchpoint.x.into(), touchpoint.y.into(), 10, 10, Rgb565::GREEN);
        }
    }

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
        let touchpanel = TouchPanel::new(twim, Some(MainTouchPanelHandler{}));

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

        display_init::spawn().unwrap();

        (Shared {
            display,
            gpiote,
            touchpanel,

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
            display.clear(Rgb565::WHITE).unwrap();
        });
        write_counter::spawn().unwrap();
    }

    #[task(shared = [display, counter])]
    fn write_counter(ctx: write_counter::Context) {
        (ctx.shared.display, ctx.shared.counter).lock(|display, counter| {
            let mut character_style = MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK);
            character_style.set_background_color(Some(Rgb565::WHITE));
            Text::with_baseline(&format!("{}", counter), Point::new(0, 0), character_style, Baseline::Top)
                .draw(display)
                .unwrap();
        });
    }

    #[task(binds = GPIOTE, shared = [gpiote, counter, touchpanel])]
    fn gpiote_interrupt(ctx: gpiote_interrupt::Context) {
        (ctx.shared.gpiote, ctx.shared.counter, ctx.shared.touchpanel).lock(|gpiote, counter, touchpanel| {
            if gpiote.channel0().is_event_triggered() {
                *counter += 1;
            } else if gpiote.channel1().is_event_triggered() {
                touchpanel.handle_interrupt();
            } else {
                panic!("Unknown channel triggered");
            }
            gpiote.reset_events()
        });
        write_counter::spawn().unwrap();
    }

    #[task(shared = [display])]
    fn draw_rectangle(mut ctx: draw_rectangle::Context, x: i32, y: i32, width: u32, height: u32, color: Rgb565) {
        ctx.shared.display.lock(|display| {
            Rectangle::new(Point::new(x, y), Size::new(width, height))
                .into_styled(PrimitiveStyle::with_fill(color))
                .draw(display)
                .unwrap();
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
