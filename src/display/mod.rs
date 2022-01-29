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

use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics_core::pixelcolor::raw::ToBytes;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{OriginDimensions, Size};
use embedded_graphics_core::Pixel;

use core::convert::Infallible;

pub struct Display<PIXEL>
where
    PIXEL: RgbColor,
{
    pin_backlight_low: Pin<Output<PushPull>>,
    pin_backlight_mid: Pin<Output<PushPull>>,
    pin_backlight_high: Pin<Output<PushPull>>,
    pin_reset: Pin<Output<PushPull>>,

    pin_command_data: Pin<Output<PushPull>>, // Low = command, High = data
    pin_chip_select: Pin<Output<PushPull>>,  // Low = enabled, High = disabled

    spi: Spim<SPIM1>,
    delay: Delay,

    // Unused, need to use PIXEL somewhere
    _background: PIXEL,
}

#[derive(Debug)]
#[repr(u8)]
pub enum RGBPixelFormat {
    Format65K = 0b101,
    Format262K = 0b110,
}

#[derive(Debug)]
#[repr(u8)]
pub enum ControlPixelFormat {
    Format12bpp = 0b011,
    Format16bpp = 0b101,
    Format18bpp = 0b110,
    Format16MTruncated = 0b111,
}

#[allow(dead_code)] // Not all commands are used, but should be implemented
#[derive(Debug)]
enum DisplayCommand {
    SleepIn,                    // Enable sleep mode
    SleepOut,                   // Disable sleep mode
    InvertOff,                  // Invert display on
    InvertOn,                   // Invert display off
    DisplayOn,                  // Power on display
    DisplayOff,                 // Power off display
    ColumnAddressSet(u16, u16), // Select column range (begin, end)
    RowAddressSet(u16, u16),    // Select row range (begin, end)
    RamWrite(Vec<u8>),          // Write data to display ram
    SoftwareReset,              // Soft-reset the system
    MemoryDataAccessControl(u8),// Control how memory is written/read
    InterfacePixelFormat(       // Set the format of the RGB data interface
        RGBPixelFormat,
        ControlPixelFormat
    ),
    NormalModeOn,               // Enable normal mode
    ReadDisplayId,              // Read the display id => ((dummy), manufacturer, version, id)
    ReadDisplayStatus,          // Read the display status => ((dummy), 4 bytes bitvectors)
}

#[derive(Debug)]
enum TransmissionByte {
    Data(u8),               // CD pin needs to be high 
    Command(u8),            // CD pin needs to be low
    CommandRead(u8, u8),    // Send a command and read response bytes
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
            DisplayCommand::ColumnAddressSet(s, e) => vec![
                TransmissionByte::Command(0x2a),
                TransmissionByte::Data((s >> 8) as u8),
                TransmissionByte::Data((s & 0xff) as u8),
                TransmissionByte::Data((e >> 8) as u8),
                TransmissionByte::Data((e & 0xff) as u8),
            ],
            DisplayCommand::RowAddressSet(s, e) => vec![
                TransmissionByte::Command(0x2b),
                TransmissionByte::Data((s >> 8) as u8),
                TransmissionByte::Data((s & 0xff) as u8),
                TransmissionByte::Data((e >> 8) as u8),
                TransmissionByte::Data((e & 0xff) as u8),
            ],
            DisplayCommand::RamWrite(v) => {
                let mut ret = vec![TransmissionByte::Command(0x2c)];
                v.iter().for_each(|x| {
                    ret.push(TransmissionByte::Data(*x));
                });
                ret
            },
            DisplayCommand::SoftwareReset => vec![
                TransmissionByte::Command(0x01)
            ],
            DisplayCommand::MemoryDataAccessControl(c) => vec![
                TransmissionByte::Command(0x36),
                TransmissionByte::Data(c),
            ],
            DisplayCommand::InterfacePixelFormat(r, c) => {
                let data = (r as u8) << 4 | (c as u8); 
                vec![
                    TransmissionByte::Command(0x3a),
                    TransmissionByte::Data(data)
                ]
            },
            DisplayCommand::NormalModeOn => vec![
                TransmissionByte::Command(0x13)
            ],
            DisplayCommand::ReadDisplayId => vec![
                TransmissionByte::CommandRead(0x04, 4)
            ],
            DisplayCommand::ReadDisplayStatus => vec![
                TransmissionByte::CommandRead(0x09, 5)
            ],
        }
    }
}

impl<
    PIXEL: RgbColor
> Display<PIXEL> {
    fn send(&mut self, command: DisplayCommand) -> Option<Vec<u8>> {
        self.pin_chip_select.set_low().unwrap();

        let parts: Vec<TransmissionByte> = command.into();
        let mut ret = vec![];

        parts.iter().for_each(|b| {
            if let Some(res) = self.transmit_byte(b) {
                ret.append(&mut res.clone());
            }
        });

        self.pin_chip_select.set_high().unwrap();

        return Some(ret);
    }
    
    fn transmit_byte(&mut self, b: &TransmissionByte) -> Option<Vec<u8>> {
        match b {
            TransmissionByte::Data(d) => {
                self.pin_command_data.set_high().unwrap();
                <Spim<SPIM1> as Write<u8>>::write(&mut self.spi, &[*d]).unwrap();
                None
            },
            TransmissionByte::Command(c) => {
                self.pin_command_data.set_low().unwrap();
                <Spim<SPIM1> as Write<u8>>::write(&mut self.spi, &[*c]).unwrap();
                None
            },
            // TODO: does not seem to work...
            TransmissionByte::CommandRead(c, n) => {
                let mut ret = vec![1; (*n).into()];
                // HACK: using pin_command_data as chip select works
                // there is (afaict) no function that allows an asymmetric r/w
                self.spi.transfer_split_uneven(
                    &mut self.pin_command_data,
                    &[*c],
                    &mut ret
                ).unwrap();
                Some(ret)
            }
        }
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

    pub fn set_normal_mode(&mut self) {
        self.send(DisplayCommand::NormalModeOn);
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

    pub fn hard_reset(&mut self) {
        self.pin_reset.set_high().unwrap(); // Make sure we are enabled
        self.pin_reset.set_low().unwrap();
        self.delay.delay_ms(120u8);
        self.pin_reset.set_high().unwrap();
    }

    // TODO: make this a nicer struct, for now, see spec
    pub fn set_memory_data_access_control(&mut self, config: u8) {
        self.send(DisplayCommand::MemoryDataAccessControl(config));
    }

    pub fn set_interface_pixel_format(
        &mut self, rgb_format: RGBPixelFormat,
        control_format: ControlPixelFormat
    ) {
        self.send(DisplayCommand::InterfacePixelFormat(rgb_format, control_format));
    }

    pub fn select_area(
        &mut self,
        start_row: u16,
        start_col: u16,
        end_row: u16,
        end_col: u16,
    ) {
        self.send(DisplayCommand::ColumnAddressSet(start_col, end_col));
        self.send(DisplayCommand::RowAddressSet(start_row, end_row));
    }

    pub fn read_id(&mut self) -> (u8, u8, u8) {
        let ret = self.send(DisplayCommand::ReadDisplayId).unwrap();

        return (ret[0], ret[1], ret[2]);
    }

    pub fn read_status(&mut self) -> (u8, u8, u8, u8) {
        let ret = self.send(DisplayCommand::ReadDisplayStatus).unwrap();

        return (ret[1], ret[2], ret[3], ret[4]);
    }

    pub fn init(&mut self) {
        self.hard_reset();

        // Should not be 120ms, but 5ms
        self.delay.delay_ms(120u8);

        self.software_reset();
        self.set_interface_pixel_format(
            RGBPixelFormat::Format65K,
            ControlPixelFormat::Format18bpp
        );
        self.set_memory_data_access_control(0b00001000);
        self.select_area(0, 0, 200, 200);

        self.set_invert(true);
        self.set_sleep(false);
        self.set_normal_mode();
        self.set_brightness(0x7);

        for _ in 0..240 {
            for _ in 0..240 {
                self.transmit_byte(&TransmissionByte::Data(0xff));
            }
            self.delay.delay_ms(10u8);
        }

        self.set_display_on(true);
    }
}

impl Display<Rgb565> {
    pub fn new(
        pin_backlight_low: Pin<Output<PushPull>>,
        pin_backlight_mid: Pin<Output<PushPull>>,
        pin_backlight_high: Pin<Output<PushPull>>,

        pin_command_data: Pin<Output<PushPull>>,
        pin_chip_select: Pin<Output<PushPull>>,
        pin_reset: Pin<Output<PushPull>>,
        spi: Spim<SPIM1>,
        delay: Delay,
    ) -> Display<Rgb565> {
        Display {
            pin_backlight_low,
            pin_backlight_mid,
            pin_backlight_high,
            pin_command_data,
            pin_chip_select,
            pin_reset,

            spi,
            delay,
            
            _background: Rgb565::new(0xff, 0, 0),
        }
    }
}

impl OriginDimensions for Display<Rgb565> {
    fn size(&self) -> Size {
        Size::new(240, 240)
    }
}

impl DrawTarget for Display<Rgb565> {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>> {
        self.set_display_on(false);
        pixels.into_iter().for_each(|p| {
            let (row, col, color) = (p.0.x.try_into().unwrap(), p.0.y.try_into().unwrap(), p.1);
            self.select_area(row, col, row, col);
            self.send(DisplayCommand::RamWrite(color.to_be_bytes().to_vec()));
        });
        self.set_display_on(true);

        Ok(())
    }
}
