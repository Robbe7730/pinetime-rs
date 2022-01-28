use rtt_target::rprintln;

use nrf52832_hal::prelude::OutputPin;

use nrf52832_hal::gpio::Pin;
use nrf52832_hal::gpio::Output;
use nrf52832_hal::gpio::PushPull;

use nrf52832_hal::spim::Spim;
use nrf52832_hal::prelude::_embedded_hal_blocking_spi_Write as Write;
use nrf52832_hal::prelude::_embedded_hal_blocking_delay_DelayMs as DelayMs;

use nrf52832_hal::pac::SPIM1;

use nrf52832_hal::delay::Delay;

use alloc::vec::Vec;
use alloc::vec;

pub struct Display<SPI>
where
    SPI: Write<u8, Error = nrf52832_hal::spim::Error>
{
    pin_backlight_low: Pin<Output<PushPull>>,
    pin_backlight_mid: Pin<Output<PushPull>>,
    pin_backlight_high: Pin<Output<PushPull>>,

    pin_command_data: Pin<Output<PushPull>>, // Low = command, High = data
    pin_chip_select: Pin<Output<PushPull>>,  // Low = enabled, High = disabled

    spi: SPI,
    delay: Delay,
}

enum DisplayCommand {
    SleepIn,                    // Enable sleep mode
    SleepOut,                   // Disable sleep mode
    InvertOff,                  // Invert display on
    InvertOn,                   // Invert display off
    DisplayOn,                  // Power on display
    DisplayOff,                 // Power off display
    SoftwareReset,              // Soft-reset the system
}

#[derive(Debug)]
enum TransmissionByte {
    Data(u8),    // CD pin needs to be high 
    Command(u8), // CD pin needs to be low
}

impl From<DisplayCommand> for Vec<TransmissionByte> {
    fn from(item: DisplayCommand) -> Self {
        match item {
            DisplayCommand::SleepIn => vec![
                TransmissionByte::Command(0x10)
            ],
            DisplayCommand::SleepOut  => vec![
                TransmissionByte::Command(0x11)
            ],
            DisplayCommand::InvertOff => vec![
                TransmissionByte::Command(0x20)
            ],
            DisplayCommand::InvertOn => vec![
                TransmissionByte::Command(0x21)
            ],
            DisplayCommand::DisplayOff => vec![
                TransmissionByte::Command(0x28)
            ],
            DisplayCommand::DisplayOn => vec![
                TransmissionByte::Command(0x29)
            ],
            DisplayCommand::SoftwareReset => vec![
                TransmissionByte::Command(0x01)
            ],
        }
    }
}

impl<T: Write<u8, Error = nrf52832_hal::spim::Error>> Display<T> {
    fn send(&mut self, command: DisplayCommand) {
        self.pin_chip_select.set_low().unwrap();

        let parts: Vec<TransmissionByte> = command.into();
        rprintln!("{:#?}", parts);
        parts.iter().for_each(|b| {
            match b {
                TransmissionByte::Data(d) => {
                    self.pin_command_data.set_high().unwrap();
                    self.spi.write(&[*d]).unwrap();
                },
                TransmissionByte::Command(c) => {
                    self.pin_command_data.set_low().unwrap();
                    self.spi.write(&[*c]).unwrap();
                }
            }
        });

        self.pin_chip_select.set_high().unwrap();
    }

    pub fn set_sleep(&mut self, value: bool) {
        if value {
            self.send(DisplayCommand::SleepIn);
        } else {
            self.send(DisplayCommand::SleepOut);
        }

        self.delay.delay_ms(5u8);
    }

    pub fn set_invert(&mut self, value: bool) {
        if value {
            self.send(DisplayCommand::InvertOn);
        } else {
            self.send(DisplayCommand::InvertOff);
        }
    }

    pub fn set_display_on(&mut self, value: bool) {
        if value {
            self.send(DisplayCommand::DisplayOn);
        } else {
            self.send(DisplayCommand::DisplayOff);
        }
    }

    pub fn set_brightness(&mut self, value: u8) {
        if (value & 0b001) != 0 {
            self.pin_backlight_low.set_low().unwrap();
        } else {
            self.pin_backlight_low.set_high().unwrap();
        }

        if (value & 0b010) != 0 {
            self.pin_backlight_mid.set_low().unwrap();
        } else {
            self.pin_backlight_mid.set_high().unwrap();
        }

        if (value & 0b100) != 0 {
            self.pin_backlight_high.set_low().unwrap();
        } else {
            self.pin_backlight_high.set_high().unwrap();
        }
    }

    pub fn software_reset(&mut self) {
        self.send(DisplayCommand::SoftwareReset);
        self.delay.delay_ms(5u8);
    }
}

impl Display<Spim<SPIM1>> {
    pub fn new(
        pin_backlight_low: Pin<Output<PushPull>>,
        pin_backlight_mid: Pin<Output<PushPull>>,
        pin_backlight_high: Pin<Output<PushPull>>,
        pin_command_data: Pin<Output<PushPull>>,
        pin_chip_select: Pin<Output<PushPull>>,
        spi: Spim<SPIM1>,
        delay: Delay,
    ) -> Display<Spim<SPIM1>> {
        Display {
            pin_backlight_low,
            pin_backlight_mid,
            pin_backlight_high,
            pin_command_data,
            pin_chip_select,

            spi,
            delay,
        }
    }
}
