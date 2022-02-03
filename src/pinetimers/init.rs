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

use alloc::boxed::Box;

use spin::Mutex;

use rubble_nrf5x::radio::BleRadio;
use rubble_nrf5x::utils::get_device_address;
use rubble_nrf5x::timer::BleTimer;
use rubble::link::{LinkLayer, Responder};
use rubble::link::ad_structure::AdStructure;
use rubble::link::queue::PacketQueue;
use rubble::l2cap::{BleChannelMap, L2CAPState};
use rubble::security::NoSecurity;
use rubble::link::queue::SimpleQueue;
use rubble::gatt::BatteryServiceAttrs;

use crate::drivers::display::Display;
use crate::drivers::timer::MonoTimer;
use crate::drivers::touchpanel::TouchPanel;
use crate::devicestate::DeviceState;
use crate::ui::screen::{Screen, ScreenMain};
use crate::drivers::battery::Battery;
use crate::drivers::flash::FlashMemory;

pub enum BluetoothConfig {}

impl rubble::config::Config for BluetoothConfig {
    type Timer = BleTimer<crate::pinetimers::BluetoothTimer>;
    type Transmitter = BleRadio;
    type ChannelMapper = BleChannelMap<BatteryServiceAttrs, NoSecurity>;
    type PacketQueue = &'static mut SimpleQueue;
}

pub struct Shared {
    pub gpiote: Gpiote,
    pub rtc: Rtc<super::ConnectedRtc>,

    pub display: Display<super::PixelType, super::ConnectedSpim>,
    pub touchpanel: TouchPanel,
    pub flash: FlashMemory,
    pub bluetooth: BleRadio,
    pub ble_ll: LinkLayer<BluetoothConfig>,
    pub ble_r: Responder<BluetoothConfig>,

    pub current_screen: Box<dyn Screen<Display<super::PixelType, super::ConnectedSpim>>>,
    pub devicestate: DeviceState,
}

pub struct Local {}

pub fn init(mut ctx: crate::tasks::init::Context) -> (Shared, Local, crate::tasks::init::Monotonics) {
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
        let display: Display<super::PixelType, super::ConnectedSpim> = Display::new(
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
        Clocks::new(ctx.device.CLOCK)
            .start_lfclk()
            .enable_ext_hfosc(); // Bluetooth needs this

        // Set up RTC
        // Prescaler value for 8Hz (125ms period)
        let rtc = Rtc::new(ctx.device.RTC1, 4095).unwrap();
        rtc.enable_counter();

        // Set up Bluetooth
        // TODO: Put this in a separate driver
        let device_address = get_device_address();
        rprintln!("{:?}", device_address);

        // Not sure what this does...
        ctx.core.DCB.enable_trace();
        ctx.core.DWT.enable_cycle_counter();
        
        let mut bluetooth = BleRadio::new(
            ctx.device.RADIO,
            &ctx.device.FICR,
            ctx.local.ble_tx_buf,
            ctx.local.ble_rx_buf,
        );

        let ble_timer = BleTimer::init(ctx.device.TIMER2);

        // Set up queues
        let (tx_prod, tx_cons) = ctx.local.ble_tx_queue.split();
        let (rx_prod, rx_cons) = ctx.local.ble_rx_queue.split();

        let mut ble_ll = LinkLayer::<BluetoothConfig>::new(device_address, ble_timer);

        let ble_r = Responder::<BluetoothConfig>::new(
            tx_prod,
            rx_cons,
            L2CAPState::new(BleChannelMap::with_attributes(BatteryServiceAttrs::new())),
        );

        let next_update = ble_ll
            .start_advertise(
                rubble::time::Duration::from_millis(200),
                &[AdStructure::CompleteLocalName("pinetime-rs")],
                &mut bluetooth,
                tx_cons,
                rx_prod,
            )
            .unwrap();
        ble_ll.timer().configure_interrupt(next_update);

        // Set up the UI
        let screen = Box::new(ScreenMain::new());

        // self_test::spawn().unwrap();

        crate::tasks::display_init::spawn().unwrap();

        (Shared {
            gpiote,
            rtc,

            display,
            touchpanel,
            flash,
            bluetooth,
            ble_ll,
            ble_r,

            current_screen: screen,
            devicestate: DeviceState::new(battery),
        }, Local {}, crate::tasks::init::Monotonics(timer0))
}
