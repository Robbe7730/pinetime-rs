use nrf52832_hal::prelude::OutputPin;
use nrf52832_hal::prelude::_embedded_hal_blocking_spi_Write as Write;
use nrf52832_hal::prelude::_embedded_hal_blocking_delay_DelayMs as DelayMs;
use nrf52832_hal::prelude::_embedded_hal_blocking_delay_DelayUs as DelayUs;
use nrf52832_hal::gpio::{Pin, Output, PushPull};
use nrf52832_hal::spim::Spim;
use nrf52832_hal::pac::SPIM0;
use nrf52832_hal::delay::Delay;

use alloc::vec::Vec;

use core::marker::PhantomData;

use commands::{DisplayCommand, RGBPixelFormat, ControlPixelFormat};

use embedded_graphics_core::pixelcolor::{Gray2, GrayColor, Rgb565, Rgb666, RgbColor};

mod commands;
mod graphics;

#[derive(Debug)]
pub enum TransmissionByte {
    Data(u8),               // CD pin needs to be high 
    Command(u8),            // CD pin needs to be low
}

pub struct Display<COLOR> {
    pin_backlight_low: Pin<Output<PushPull>>,   // Set backlight brightness (3 bits)
    pin_backlight_mid: Pin<Output<PushPull>>,
    pin_backlight_high: Pin<Output<PushPull>>,

    pin_reset: Pin<Output<PushPull>>,           // Low (10 µs) = reset, high = running

    pin_command_data: Pin<Output<PushPull>>,    // Low = command, High = data
    pin_chip_select: Pin<Output<PushPull>>,     // Low = enabled, High = disabled

    spi: Spim<SPIM0>,   // Serial interface (has no read!)
    delay: Delay,       // Delay source

    _pixel: PhantomData<COLOR>,
}

pub trait DisplaySupported<COLOR> {
    fn set_pixel_config(&mut self);
    fn transmit_color(&mut self, color: COLOR);
}

impl<COLOR> Display<COLOR>
where
    Display<COLOR>: DisplaySupported<COLOR>
{
    fn send(&mut self, command: DisplayCommand) {
        self.pin_chip_select.set_low().unwrap();

        self.send_no_cs(command);

        self.pin_chip_select.set_high().unwrap();
    }

    // Send without changing Chip Select, useful for StartRamWrite
    fn send_no_cs(&mut self, command: DisplayCommand) {
        let parts: Vec<TransmissionByte> = command.into();

        parts.iter().for_each(|b| {
            self.transmit_byte(b)
        });
    }
    
    fn transmit_byte(&mut self, b: &TransmissionByte) {
        match b {
            TransmissionByte::Data(d) => {
                self.pin_command_data.set_high().unwrap();
                <Spim<SPIM0> as Write<u8>>::write(&mut self.spi, &[*d]).unwrap();
            },
            TransmissionByte::Command(c) => {
                self.pin_command_data.set_low().unwrap();
                <Spim<SPIM0> as Write<u8>>::write(&mut self.spi, &[*c]).unwrap();
            },
        }
    }

    fn set_sleep(&mut self, value: bool) {
        if value {
            self.send(DisplayCommand::SleepIn);
        } else {
            self.send(DisplayCommand::SleepOut);
        }

        self.delay.delay_ms(5u8);
    }

    fn set_invert(&mut self, value: bool) {
        if value {
            self.send(DisplayCommand::InvertOn);
        } else {
            self.send(DisplayCommand::InvertOff);
        }
    }

    fn set_display_on(&mut self, value: bool) {
        if value {
            self.send(DisplayCommand::DisplayOn);
        } else {
            self.send(DisplayCommand::DisplayOff);
        }
    }

    fn set_normal_mode(&mut self) {
        self.send(DisplayCommand::NormalModeOn);
    }

    fn set_brightness(&mut self, value: u8) {
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

    fn software_reset(&mut self) {
        self.send(DisplayCommand::SoftwareReset);
        // Shouldn't be 120, but 5 ms
        self.delay.delay_ms(120u8);
    }

    fn hard_reset(&mut self) {
        self.pin_reset.set_low().unwrap();
        self.delay.delay_us(10u8);
        self.pin_reset.set_high().unwrap();
    }

    // TODO: make this a nicer struct, for now, see spec
    fn set_memory_data_access_control(&mut self, config: u8) {
        self.send(DisplayCommand::MemoryDataAccessControl(config));
    }

    fn set_interface_pixel_format(
        &mut self, rgb_format: RGBPixelFormat,
        control_format: ControlPixelFormat
    ) {
        self.send(DisplayCommand::InterfacePixelFormat(rgb_format, control_format));
    }

    fn select_area(
        &mut self,
        start_row: u16,
        start_col: u16,
        end_row: u16,
        end_col: u16,
    ) {
        self.send(DisplayCommand::ColumnAddressSet(start_col, end_col));
        self.send(DisplayCommand::RowAddressSet(start_row, end_row));
    }

    pub fn init(&mut self) {
        self.pin_reset.set_high().unwrap();
        
        self.hard_reset();

        self.software_reset();

        self.set_sleep(false);

        // self.set_invert(false);

        self.set_pixel_config();

        self.select_area(0, 0, 240, 240);
        
        self.set_invert(true);

        self.set_normal_mode();

        self.set_display_on(true);

        self.set_brightness(0x7);

    }

    pub fn new(
        pin_backlight_low: Pin<Output<PushPull>>,
        pin_backlight_mid: Pin<Output<PushPull>>,
        pin_backlight_high: Pin<Output<PushPull>>,

        pin_command_data: Pin<Output<PushPull>>,
        pin_chip_select: Pin<Output<PushPull>>,
        pin_reset: Pin<Output<PushPull>>,
        spi: Spim<SPIM0>,
        delay: Delay,
    ) -> Display<COLOR> {
        Self {
            pin_backlight_low,
            pin_backlight_mid,
            pin_backlight_high,

            pin_reset,

            pin_command_data,
            pin_chip_select,

            spi,
            delay,

            _pixel: PhantomData
        }
    }
}

impl DisplaySupported<Rgb565> for Display<Rgb565> {
    fn set_pixel_config(&mut self) {
        self.set_interface_pixel_format(
            RGBPixelFormat::Format65K,
            ControlPixelFormat::Format16bpp
        );

        self.set_memory_data_access_control(0b00000000);
    }

    // rrrrrggg gggbbbbb
    fn transmit_color(&mut self, pixel: Rgb565) {
        let byte1 = (pixel.r() << 3) | ((pixel.g() >> 3) & 0b111);
        let byte2 = (pixel.g() << 5) | (pixel.b() & 0b11111);

        self.transmit_byte(&TransmissionByte::Data(byte1));
        self.transmit_byte(&TransmissionByte::Data(byte2));
    }
}

impl DisplaySupported<Rgb666> for Display<Rgb666> {
    fn set_pixel_config(&mut self) {
        self.set_interface_pixel_format(
            RGBPixelFormat::Format262K,
            ControlPixelFormat::Format18bpp
        );

        self.set_memory_data_access_control(0b00000000);
    }

    // rrrrrr00 gggggg00 bbbbbb00
    fn transmit_color(&mut self, pixel: Rgb666) {
        self.transmit_byte(&TransmissionByte::Data(pixel.r() << 2));
        self.transmit_byte(&TransmissionByte::Data(pixel.g() << 2));
        self.transmit_byte(&TransmissionByte::Data(pixel.b() << 2));
    }
}

impl DisplaySupported<Gray2> for Display<Gray2> {
    fn set_pixel_config(&mut self) {
        self.set_interface_pixel_format(
            RGBPixelFormat::Format262K,
            ControlPixelFormat::Format18bpp
        );

        self.set_memory_data_access_control(0b00000000);
    }

    // using 666
    fn transmit_color(&mut self, pixel: Gray2) {
        let byte = match pixel.luma() {
            0b00 => 0x00,
            0b01 => 0x0f,
            0b10 => 0xf0,
            0b11 => 0xff,
            x => panic!("Invalid luma {}", x),
        };
        self.transmit_byte(&TransmissionByte::Data(byte << 2));
        self.transmit_byte(&TransmissionByte::Data(byte << 2));
        self.transmit_byte(&TransmissionByte::Data(byte << 2));
    }
}
