use rtt_target::{rtt_init_print, rprintln};

use nrf52832_hal::gpiote::Gpiote;
use nrf52832_hal::gpio::{Level, p0};
use nrf52832_hal::spim::{self, MODE_3, Spim};
use nrf52832_hal::twim::{self, Twim};
use nrf52832_hal::delay::Delay;
use nrf52832_hal::saadc::{Saadc, SaadcConfig};
use nrf52832_hal::clocks::Clocks;
use nrf52832_hal::pac::TIMER0;
use nrf52832_hal::rtc::Rtc;
use nrf52832_hal::wdt::{Watchdog, count, WatchdogHandle};
use nrf52832_hal::wdt::handles::HdlN;

use alloc::boxed::Box;

use spin::Mutex;

use crate::drivers::display::Display;
use crate::drivers::timer::MonoTimer;
use crate::drivers::touchpanel::TouchPanel;
use crate::ui::screen::{Screen, ScreenMain};
use crate::drivers::battery::Battery;
use crate::drivers::flash::{InternalFlash, ExternalFlash};
use crate::drivers::bluetooth::Bluetooth;
use crate::pinetimers::{PixelType, ConnectedSpim, ConnectedRtc};
use crate::drivers::clock::Clock;
use crate::drivers::mcuboot::MCUBoot;

pub struct Shared {
    pub gpiote: Gpiote,
    pub watchdog_handles: [WatchdogHandle<HdlN> ; 1],

    pub display: Display<PixelType, ConnectedSpim>,
    pub touchpanel: TouchPanel,
    pub external_flash: ExternalFlash,
    pub internal_flash: InternalFlash,
    pub bluetooth: Bluetooth,
    pub battery: Battery,
    pub clock: Clock<ConnectedRtc>,
    pub mcuboot: MCUBoot,

    pub current_screen: Box<dyn Screen<Display<PixelType, ConnectedSpim>>>,
}

pub struct Local {}

pub fn init(ctx: crate::tasks::init::Context) -> (Shared, Local, crate::tasks::init::Monotonics) {
        rtt_init_print!();
        rprintln!("Pijn tijd");

        unsafe {
            // Set up heap
            let heap_start = 0x2000_1000;
            let heap_end = 0x2001_0000;
            crate::HEAP.lock().init(heap_start, heap_end - heap_start);
        }

        // Set up watchdog (enabled by MCUBoot)
        let watchdog = Watchdog::try_recover::<count::One>(ctx.device.WDT).unwrap();
        let (watchdog_handle_0,) = watchdog.handles;
        let watchdog_handles = [watchdog_handle_0.degrade()];

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
        let display: Display<PixelType, ConnectedSpim> = Display::new(
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

        // Set up external flash
        let external_flash = ExternalFlash::new(
            ctx.local.spi_lock,
            gpio.p0_05.into_push_pull_output(Level::High).degrade(),
        );

        // Set up internal flash
        let mut internal_flash = InternalFlash::new(
            ctx.device.NVMC
        );

        let mcuboot = MCUBoot::get(&mut internal_flash);

        // Enable LFCLK
        Clocks::new(ctx.device.CLOCK)
            .start_lfclk();

        // Set up RTC
        // Prescaler value for 8Hz (125ms period)
        let rtc = Rtc::new(ctx.device.RTC1, 4095).unwrap();
        rtc.enable_counter();
        let clock = Clock::new(rtc);

        // Set up Bluetooth
        let bluetooth = Bluetooth::new(
            ctx.device.RADIO,
            ctx.device.FICR,
            ctx.local.ble_packet_buffer
        );

        // Set up the UI
        let screen = Box::new(ScreenMain::new());

        crate::tasks::pet_watchdog::spawn().unwrap();
        crate::tasks::validate::spawn().unwrap();

        (Shared {
            gpiote,
            watchdog_handles,

            display,
            touchpanel,
            external_flash,
            internal_flash,
            bluetooth,
            battery,
            clock,
            mcuboot,

            current_screen: screen,
        }, Local {}, crate::tasks::init::Monotonics(timer0))
}
