use rtt_target::rprintln;

use nrf52832_hal::prelude::OutputPin;

use nrf52832_hal::gpio::Pin;
use nrf52832_hal::gpio::Output;
use nrf52832_hal::gpio::PushPull;

use nrf52832_hal::spim::Spim;

use nrf52832_hal::pac::SPIM1;

pub struct Display {
    pin_backlight_low: Pin<Output<PushPull>>,
    pin_backlight_mid: Pin<Output<PushPull>>,
    pin_backlight_high: Pin<Output<PushPull>>,
    pin_command_data: Pin<Output<PushPull>>, // Low = command, High = data
    pin_chip_select: Pin<Output<PushPull>>,  // Low = enabled, High = disabled

    spi: Spim<SPIM1>,
}

#[repr(u8)]
enum DisplayCommand {
    SleepIn = 0x10,
    SleepOut = 0x11,
    InvertOff = 0x20,
    InvertOn = 0x21,
    DisplayOn = 0x29,
    DisplayOff = 0x28,
}

impl Display {
    fn send_command(&mut self, command: DisplayCommand) {
        self.pin_command_data.set_low().unwrap();

        let value: u8 = command as u8;
        rprintln!("{:x}", value);
        self.spi.write(&mut self.pin_chip_select, &[value]).unwrap();
    }

    pub fn set_sleep(&mut self, value: bool) {
        if value {
            self.send_command(DisplayCommand::SleepIn);
        } else {
            self.send_command(DisplayCommand::SleepOut);
        }
    }

    pub fn set_invert(&mut self, value: bool) {
        if value {
            self.send_command(DisplayCommand::InvertOn);
        } else {
            self.send_command(DisplayCommand::InvertOff);
        }
    }

    pub fn set_display_on(&mut self, value: bool) {
        if value {
            self.send_command(DisplayCommand::DisplayOn);
        } else {
            self.send_command(DisplayCommand::DisplayOff);
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

    pub fn new(
        pin_backlight_low: Pin<Output<PushPull>>,
        pin_backlight_mid: Pin<Output<PushPull>>,
        pin_backlight_high: Pin<Output<PushPull>>,
        pin_command_data: Pin<Output<PushPull>>,
        pin_chip_select: Pin<Output<PushPull>>,
        spi: Spim<SPIM1>,
    ) -> Display {
        Display {
            pin_backlight_low,
            pin_backlight_mid,
            pin_backlight_high,
            pin_command_data,
            pin_chip_select,

            spi,
        }
    }
}
