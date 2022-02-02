#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

mod drivers;
mod ui;
mod devicestate;

extern crate alloc;

#[rtic::app(device = nrf52832_hal::pac, dispatchers = [SWI0_EGU0])]
mod pinetimers {
    use crate::drivers::timer::MonoTimer;
    use crate::drivers::display::Display;
    use crate::drivers::touchpanel::TouchPanel;
    use crate::drivers::battery::Battery;
    use crate::drivers::flash::FlashMemory;

    use crate::devicestate::DeviceState;

    use crate::ui::screen::{Screen, ScreenMain};

    use rtt_target::{rprintln, rtt_init_print};

    use nrf52832_hal::pac::{TIMER0, SPIM0, RTC1};
    use nrf52832_hal::gpio::{Level, p0};
    use nrf52832_hal::gpiote::Gpiote;
    use nrf52832_hal::spim::{self, MODE_3, Spim};
    use nrf52832_hal::twim::{self, Twim};
    use nrf52832_hal::delay::Delay;
    use nrf52832_hal::saadc::{Saadc, SaadcConfig};
    use nrf52832_hal::rtc::{Rtc, RtcInterrupt};
    use nrf52832_hal::clocks::Clocks;

    use chrono::Duration;

    use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
    use embedded_graphics_core::draw_target::DrawTarget;

    use alloc::boxed::Box;

    use spin::Mutex;

    use fugit::ExtU32;

    #[monotonic(binds = TIMER0, default = true)]
    type Mono0 = MonoTimer<TIMER0>;

    type DisplayColor = Rgb565;
    type ConnectedSpim = SPIM0;
    type ConnectedRtc = RTC1;

    #[shared]
    struct Shared {
        gpiote: Gpiote,
        rtc: Rtc<ConnectedRtc>,

        display: Display<DisplayColor, ConnectedSpim>,
        touchpanel: TouchPanel,
        flash: FlashMemory,

        current_screen: Box<dyn Screen<Display<DisplayColor, ConnectedSpim>>>,
        devicestate: DeviceState,
    }

    #[local]
    struct Local {}

    #[init(local = [spi_lock: Mutex<Option<Spim<ConnectedSpim>>> = Mutex::new(None)])]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();
        rprintln!("Pijn tijd");

        // Set up heap
        unsafe {
            let heap_start = 0x2000_1000;
            let heap_end = 0x2001_0000;
            crate::HEAP.lock().init(heap_start, heap_end - heap_start);
        }

        let gpio = p0::Parts::new(ctx.device.P0);

        let timer0: MonoTimer<TIMER0> = MonoTimer::new(ctx.device.TIMER0);

        // Set up GPIOTE
        let gpiote = Gpiote::new(ctx.device.GPIOTE);

        // Set up SAADC
        let saadc_config = SaadcConfig::default();
        let saadc = Saadc::new(ctx.device.SAADC, saadc_config);

        // Set up button
        gpio.p0_15.into_push_pull_output(Level::High);
        let button_input_pin = gpio.p0_13.into_floating_input().degrade();

        // Fire event on button press
        gpiote.channel0()
            .input_pin(&button_input_pin)
            .lo_to_hi()
            .enable_interrupt();

        // Set up charging
        let charging_input_pin = gpio.p0_19.into_floating_input().degrade();

        // Fire event on charging state change
        gpiote.channel2()
            .input_pin(&charging_input_pin)
            .toggle()
            .enable_interrupt();

        let battery = Battery::new(
            // Charge indicator pin
            charging_input_pin,

            // Voltage pin (don't degrade because we need the typecheck if the
            // pin can be analog)
            gpio.p0_31.into_floating_input(),
            saadc,
        );

        // Set up SPI
        let spi_pins = spim::Pins {
            sck: gpio.p0_02.into_push_pull_output(Level::Low).degrade(),
            mosi: Some(gpio.p0_03.into_push_pull_output(Level::Low).degrade()),
            // MISO is not connected for the LCD, but is for flash memory
            miso: Some(gpio.p0_04.into_floating_input().degrade())
        };
        let spi = Spim::new(
            ctx.device.SPIM0,
            spi_pins,
            spim::Frequency::M8,
            MODE_3,
            0
        );
        *ctx.local.spi_lock = Mutex::new(Some(spi));

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
        let display: Display<DisplayColor, ConnectedSpim> = Display::new(
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

            ctx.local.spi_lock,
            Delay::new(ctx.core.SYST),
        );

        // Set up flash
        let flash = FlashMemory::new(
            ctx.local.spi_lock,
            gpio.p0_05.into_push_pull_output(Level::High).degrade(),
        );

        // Enable LFCLK
        let clocks = Clocks::new(ctx.device.CLOCK);
        clocks.start_lfclk();

        // Set up RTC
        // Prescaler value for 8Hz (125ms period)
        let rtc = Rtc::new(ctx.device.RTC1, 4095).unwrap();
        rtc.enable_counter();

        // Set up the UI
        let screen = Box::new(ScreenMain::new());

        // self_test::spawn().unwrap();

        display_init::spawn().unwrap();
        periodic_update_device_state::spawn_after(5.secs()).unwrap();

        (Shared {
            gpiote,
            rtc,

            display,
            touchpanel,
            flash,

            current_screen: screen,
            devicestate: DeviceState::new(battery),
        }, Local {}, init::Monotonics(timer0))
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[task(shared = [display])]
    fn display_init(mut ctx: display_init::Context) {
        ctx.shared.display.lock(|display| {
            display.init();
            display.clear(RgbColor::WHITE).unwrap();
        });
        draw_screen::spawn().unwrap();
    }

    #[task(binds = GPIOTE, shared = [gpiote, touchpanel, current_screen, devicestate])]
    fn gpiote_interrupt(ctx: gpiote_interrupt::Context) {
        (
            ctx.shared.gpiote,
            ctx.shared.touchpanel,
            ctx.shared.current_screen,
            ctx.shared.devicestate
        ).lock(|gpiote, touchpanel, current_screen, devicestate| {
            if gpiote.channel0().is_event_triggered() {
                devicestate.counter += 1;
            } else if gpiote.channel1().is_event_triggered() {
                touchpanel.handle_interrupt(Some(current_screen.get_event_handler()));
            } else if gpiote.channel2().is_event_triggered() {
                devicestate.update_battery();
            } else {
                panic!("Unknown channel triggered");
            }
            gpiote.reset_events()
        });
        draw_screen::spawn().unwrap();
    }

    #[task(shared = [devicestate, rtc])]
    fn periodic_update_device_state(ctx: periodic_update_device_state::Context) {
        //periodic_update_device_state::spawn_after(1.secs()).unwrap();
        periodic_update_device_state::spawn_after(500.millis()).unwrap();

        (
            ctx.shared.devicestate,
            ctx.shared.rtc,
        ).lock(|devicestate, rtc| {
            devicestate.update_battery();
            let new_counter = rtc.get_counter().try_into().unwrap();
            devicestate.datetime = devicestate.datetime + Duration::milliseconds(
                125i64 * (new_counter - devicestate.counter) as i64
            );
            devicestate.counter = new_counter;
        });

        draw_screen::spawn().unwrap();
    }

    #[task(shared = [display, current_screen, devicestate])]
    fn draw_screen(ctx: draw_screen::Context) {
        (
            ctx.shared.display,
            ctx.shared.current_screen,
            ctx.shared.devicestate
        ).lock(|display, current_screen, devicestate| {
            current_screen.draw(display, devicestate);
        });
    }

    #[task(shared = [flash])]
    fn self_test(mut ctx: self_test::Context) {
        ctx.shared.flash.lock(|flash| {
            flash.self_test().unwrap();
        });
    }

    #[task(shared = [current_screen])]
    fn transition(mut ctx: transition::Context, new_screen: Box<dyn Screen<Display<DisplayColor, ConnectedSpim>>>) {
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
fn on_oom(layout: Layout) -> ! {
    rprintln!("----- OOM -----");
    rprintln!("{:#?}", layout);
    loop {
        cortex_m::asm::bkpt();
    }
}
