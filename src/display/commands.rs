use alloc::vec;
use alloc::vec::Vec;

use crate::display::TransmissionByte;

#[derive(Debug)]
#[repr(u8)]
pub enum RGBPixelFormat {
    Format4K = 0b011,       // 4K RGB Interface (used in RGB444)
    Format65K = 0b101,      // 65K RGB Interface (used in RGB565)
    Format262K = 0b110,     // 262K RGB Interface (used in RGB666)
}

#[derive(Debug)]
#[repr(u8)]
pub enum ControlPixelFormat {
    Format12bpp = 0b011,        // Used in RGB444
    Format16bpp = 0b101,        // Used in RGB565
    Format18bpp = 0b110,        // Used in RGB666
    Format16MTruncated = 0b111, // Unused?
}

#[allow(dead_code)] // Not all commands are used, but should be implemented
#[derive(Debug)]
pub enum DisplayCommand {
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

