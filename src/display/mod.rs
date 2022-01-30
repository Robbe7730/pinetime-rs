// DEBUG LOG
// Writing Commands works (INVON/INVOFF)
// Writing Data works (COLMOD RGB -> BGR)
// Reading Data does not work (RDDID: always (0, 0, 0), even if vec initialized to 1s)
//   -> MISO pin is not connected to display...

use nrf52832_hal::prelude::OutputPin;

use nrf52832_hal::gpio::Pin;
use nrf52832_hal::gpio::Output;
use nrf52832_hal::gpio::PushPull;

use nrf52832_hal::spim::Spim;
use nrf52832_hal::prelude::_embedded_hal_blocking_spi_Write as Write;
use nrf52832_hal::prelude::_embedded_hal_blocking_delay_DelayMs as DelayMs;
use nrf52832_hal::prelude::_embedded_hal_blocking_delay_DelayUs as DelayUs;

use nrf52832_hal::pac::SPIM0;

use nrf52832_hal::delay::Delay;

use alloc::vec::Vec;
use alloc::vec;

use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics_core::pixelcolor::raw::ToBytes;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{OriginDimensions, Size, Dimensions};
use embedded_graphics_core::primitives::Rectangle;
use embedded_graphics_core::prelude::PointsIter;
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

    spi: Spim<SPIM0>,
    delay: Delay,

    // Unused, need to use PIXEL somewhere
    _background: PIXEL,
}

#[derive(Debug)]
#[repr(u8)]
pub enum RGBPixelFormat {
    Format65K = 0b101,      // 65K RGB Interface
    Format262K = 0b110,     // 262K RGB Interface
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
    StartRamWrite,              // Initate write to display ram
    RamWrite(Vec<u8>),          // Write data to display ram (both StartRamWrite and data write)
    SoftwareReset,              // Soft-reset the system
    MemoryDataAccessControl(u8),// Control how memory is written/read
    InterfacePixelFormat(       // Set the format of the RGB data interface
        RGBPixelFormat,
        ControlPixelFormat
    ),
    NormalModeOn,               // Enable normal mode
    WriteBrightness(u8),        // Set the brightness
}

#[derive(Debug)]
enum TransmissionByte {
    Data(u8),               // CD pin needs to be high 
    Command(u8),            // CD pin needs to be low
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
            DisplayCommand::StartRamWrite => vec![
                TransmissionByte::Command(0x2c),
            ],
            DisplayCommand::RamWrite(v) => {
                let mut ret: Vec<TransmissionByte> = DisplayCommand::StartRamWrite.into();
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
            DisplayCommand::WriteBrightness(b) => vec![
                TransmissionByte::Command(0x51),
                TransmissionByte::Data(b),
            ],
        }
    }
}

impl<
    PIXEL: RgbColor
> Display<PIXEL> {
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
        // Shouldn't be 120, but 5 ms
        self.delay.delay_ms(120u8);
    }

    pub fn hard_reset(&mut self) {
        self.pin_reset.set_low().unwrap();
        self.delay.delay_us(10u8);
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

    pub fn init(&mut self) {
        self.pin_reset.set_high().unwrap();
        
        self.hard_reset();

        self.software_reset();

        self.set_sleep(false);

        // self.set_invert(false);

        self.set_interface_pixel_format(
            RGBPixelFormat::Format65K,
            ControlPixelFormat::Format16bpp
        );

        self.set_memory_data_access_control(0b00000000);

        self.select_area(0, 0, 240, 240);
        
        self.set_invert(true);

        self.set_normal_mode();

        self.set_display_on(true);

        self.set_brightness(0x7);

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
        spi: Spim<SPIM0>,
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
        pixels.into_iter().for_each(|p| {
            if self.bounding_box().contains(p.0) {
                let (col, row, color) = (
                    p.0.x.try_into().unwrap(),
                    p.0.y.try_into().unwrap(),
                    p.1
                );
                self.select_area(row, col, row+1, col+1);
                self.send(DisplayCommand::RamWrite(color.to_be_bytes().to_vec()));
            }
        });

        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let drawable_area = area.intersection(&self.bounding_box());
        self.select_area(
            drawable_area.rows().start.try_into().unwrap(),
            drawable_area.columns().start.try_into().unwrap(),
            u16::try_from(drawable_area.rows().end).unwrap() - 1,
            u16::try_from(drawable_area.columns().end).unwrap() - 1,
        );

        self.pin_chip_select.set_low().unwrap();

        self.send_no_cs(DisplayCommand::StartRamWrite);
        area.points()
            .zip(colors)
            .for_each(|(p, c)| {
                if self.bounding_box().contains(p) {
                    let bytes = c.to_be_bytes();
                    self.transmit_byte(&TransmissionByte::Data(bytes[0]));
                    self.transmit_byte(&TransmissionByte::Data(bytes[1]));
                }
            });

        self.pin_chip_select.set_high().unwrap();

        Ok(())
    }
}
