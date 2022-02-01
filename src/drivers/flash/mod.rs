use nrf52832_hal::spim::Spim;
use nrf52832_hal::pac::SPIM0;
use nrf52832_hal::gpio::{Pin, Output, PushPull};

use alloc::vec::Vec;
use alloc::vec;

use spin::Mutex;

pub struct FlashMemory {
    // Spi can be 'static because it is accessible as long as the device is
    // powered on.
    spi: &'static Mutex<Option<Spim<SPIM0>>>,

    pin_chip_select: Pin<Output<PushPull>>,
}

enum FlashCommand {
    WriteEnable,
    WriteDisable,
    Read(u32),
    Write(u32, Vec<u8>),
    ReadIdentification,
    ReadStatusRegister0,
    ReadStatusRegister1,
    ChipErase,
    ResetEnable,
    Reset,
}

#[derive(Debug)]
pub struct FlashIdentification {
    pub manufacturer: u8,   // = 0x0b (XTX)
    pub memory_type: u8,    // = 0x15 (?)
    pub capacity: u8,       // = 0x40 (4MB?)
}

#[derive(Debug)]
pub struct FlashStatusRegisters {
    pub write_in_progress: bool,    // if true, write commands are ignored
    pub write_enable: bool,         // if false, write commands are ignored
    pub block_protect_bits: u8,     // (see spec table 1.0)
    pub status_register_protect: FlashStatusRegisterProtection,
    pub quad_enabled: bool,         // Should never be true
    pub one_time_program: bool,     // If true, status register is locked using OTP
    pub cmp: bool                   // Used with block protect bits
}

#[derive(Debug)]
pub enum FlashStatusRegisterProtection {
    SoftwareProtected,      // Status Register is writable
    HardwareProtected,      // Depending on the WP# hardware pin

    // Following 2 are on request, and thus probably not available in
    // PineTime flash memory
    PowerSupplyLockDown,    // Status Register cannot be writen until power-down power-up
    OneTimeProgram,         // Status Register is permanently protected
}

impl From<u16> for FlashStatusRegisterProtection {
    fn from(value: u16) -> FlashStatusRegisterProtection {
        match value {
            0b00 => FlashStatusRegisterProtection::SoftwareProtected,
            0b01 => FlashStatusRegisterProtection::HardwareProtected,
            0b10 => FlashStatusRegisterProtection::PowerSupplyLockDown,
            0b11 => FlashStatusRegisterProtection::OneTimeProgram,
            _ => unreachable!(),
        }
    }
}

impl From<FlashCommand> for Vec<u8> {
    fn from(command: FlashCommand) -> Vec<u8> {
        match command {
            FlashCommand::WriteEnable => vec![0x06],
            FlashCommand::WriteDisable => vec![0x04],
            FlashCommand::Read(a) => vec![
                0x03,
                ((a >> 16) & 0xff).try_into().unwrap(),
                ((a >>  8) & 0xff).try_into().unwrap(),
                ((a      ) & 0xff).try_into().unwrap(),
            ],
            FlashCommand::Write(a, v) => {
                let mut ret = vec![
                    0x02,
                    ((a >> 16) & 0xff).try_into().unwrap(),
                    ((a >>  8) & 0xff).try_into().unwrap(),
                    ((a      ) & 0xff).try_into().unwrap(),
                ];
                ret.extend(v);
                ret
            },
            FlashCommand::ReadIdentification => vec![0x9f],
            FlashCommand::ReadStatusRegister0 => vec![0x05],
            FlashCommand::ReadStatusRegister1 => vec![0x35],
            FlashCommand::ChipErase => vec![0xc7], // TODO: confirm this is 0x60 or 0xc7
            FlashCommand::ResetEnable => vec![0x66],
            FlashCommand::Reset => vec![0x99],
        }
    }
}

impl FlashMemory {
    pub fn new(
        spi: &'static Mutex<Option<Spim<SPIM0>>>,
        pin_chip_select: Pin<Output<PushPull>>
    ) -> FlashMemory {
        FlashMemory {
            spi,
            pin_chip_select
        }
    }

    fn transfer(&mut self, command: FlashCommand, rx_size: u32) -> Vec<u8> {
        // Using try_lock instead of lock() to avoid deadlocks

        // If this panics, you probably used both flash and display
        // at the same time
        let mut spi_lock = self.spi.try_lock().unwrap();
        let spi = (*spi_lock).as_mut().unwrap();

        let tx_buffer: Vec<u8> = command.into();
        let mut rx_buffer = vec![0; tx_buffer.len() + (rx_size as usize)];

        spi.transfer_split_uneven(
            &mut self.pin_chip_select,
            &tx_buffer,
            &mut rx_buffer,
        ).unwrap();

        rx_buffer.split_off(tx_buffer.len())
    }

    fn send(&mut self, command: FlashCommand) {
        // Using try_lock instead of lock() to avoid deadlocks

        // If this panics, you probably used both flash and display
        // at the same time
        let mut spi_lock = self.spi.try_lock().unwrap();
        let spi = (*spi_lock).as_mut().unwrap();

        let tx_buffer: Vec<u8> = command.into();

        spi.write(
            &mut self.pin_chip_select,
            &tx_buffer,
        ).unwrap();
    }

    fn set_write_enable(&mut self, value: bool) {
        if value {
            self.send(FlashCommand::WriteEnable);
        } else {
            self.send(FlashCommand::WriteDisable);
        }
    }

    pub fn read_status_registers(&mut self) -> FlashStatusRegisters {
        let buffer0 = self.transfer(FlashCommand::ReadStatusRegister0, 1);
        let buffer1 = self.transfer(FlashCommand::ReadStatusRegister1, 1);

        let value: u16 = ((buffer1[0] as u16) << 8) | (buffer0[0] as u16);

        FlashStatusRegisters{
            write_in_progress:       ((value      ) & 0b00001) == 1,
            write_enable:            ((value >>  1) & 0b00001) == 1,
            block_protect_bits:      ((value >>  2) & 0b11111).try_into().unwrap(),
            status_register_protect: ((value >>  7) & 0b00011).into(),
            quad_enabled:            ((value >>  9) & 0b00001) == 1,
            one_time_program:        ((value >> 10) & 0b00001) == 1,
            cmp:                     ((value >> 14) & 0b00001) == 1,
        }
    }

    pub fn full_reset(&mut self) {
        self.send(FlashCommand::ResetEnable);
        self.send(FlashCommand::Reset);
    }

    pub fn read_identification(&mut self) -> FlashIdentification {
        let buffer = self.transfer(FlashCommand::ReadIdentification, 3);

        FlashIdentification {
            manufacturer: buffer[0],
            memory_type: buffer[1],
            capacity: buffer[2],
        }
    }

    pub fn chip_erase(&mut self) {
        self.set_write_enable(true);

        self.send(FlashCommand::ChipErase);

        while self.read_status_registers().write_in_progress {}

        // Write Enable gets reset automatically
    }

    // Read flash memory starting from address `start` with length `len`
    pub fn read(&mut self, start: u32, len: u32) -> Vec<u8> {
        self.transfer(FlashCommand::Read(start), len)
    }

    // Write contents of `buffer` to address `start` (blocking)
    pub fn write(&mut self, start: u32, buffer: &[u8]) {
        self.set_write_enable(true);

        self.send(FlashCommand::Write(start, buffer.to_vec()));

        while self.read_status_registers().write_in_progress {}

        // Write Enable gets reset automatically
    }
}
