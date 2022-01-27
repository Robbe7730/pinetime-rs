use rtt_target::rprintln;

use nrf52832_hal::prelude::OutputPin;

use nrf52832_hal::gpio::Pin;
use nrf52832_hal::gpio::Output;
use nrf52832_hal::gpio::PushPull;

pub struct Display {
    sleep: bool,
    brightness: u8,
    pin_backlight_low: Pin<Output<PushPull>>,
    pin_backlight_mid: Pin<Output<PushPull>>,
    pin_backlight_high: Pin<Output<PushPull>>,
}

#[repr(u8)]
enum DisplayCommand {
    SleepIn = 0x10,
    SleepOut = 0x11,
}

impl Display {
    fn send_command(&mut self, command: DisplayCommand) {
        let value: u8 = command as u8;
        rprintln!("DISPLAY SEND {}", value);
    }

    pub fn set_sleep(&mut self, value: bool) {
        self.send_command(if value { DisplayCommand::SleepIn } else { DisplayCommand::SleepOut });

        self.sleep = value;
    }

    pub fn set_brightness(&mut self, value: u8) {
        if (value & 0b001) == 0 {
            self.pin_backlight_low.set_low().unwrap();
        } else {
            self.pin_backlight_low.set_high().unwrap();
        }

        if (value & 0b010) == 0 {
            self.pin_backlight_mid.set_low().unwrap();
        } else {
            self.pin_backlight_mid.set_high().unwrap();
        }

        if (value & 0b100) == 0 {
            self.pin_backlight_high.set_low().unwrap();
        } else {
            self.pin_backlight_high.set_high().unwrap();
        }

        self.brightness = value;
    }

    pub fn inc_brightness(&mut self) {
        self.set_brightness(self.brightness + 1);
    }

    pub fn new(
        pin_backlight_low: Pin<Output<PushPull>>,
        pin_backlight_mid: Pin<Output<PushPull>>,
        pin_backlight_high: Pin<Output<PushPull>>
    ) -> Display {
        Display {
            sleep: true,
            brightness: 0,
            pin_backlight_low,
            pin_backlight_mid,
            pin_backlight_high,
        }
    }
}
