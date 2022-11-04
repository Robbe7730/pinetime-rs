pub mod phy;
mod link;

use nrf52832_hal::pac::{RADIO, FICR};

use alloc::vec;

use self::phy::{PhyRadio, Channel, packets::{BluetoothPacket, AdvData}};

use crate::alloc::string::ToString;

pub struct Bluetooth {
    pub phy: PhyRadio,
}

impl Bluetooth {
    pub fn new(
        radio: RADIO,
        ficr: FICR,
        packet_buffer: &'static mut [u8; 258]
    ) -> Self {
        let mut phy = PhyRadio::new(radio, ficr, packet_buffer);

        phy.queue_packet(
            BluetoothPacket::AdvInd(
                phy::packets::BluetoothAddress::Public(
                    [0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12]
                ),
                vec![
                    AdvData::CompleteLocalName("It works?".to_string()),
                    AdvData::Flags(0b00000000),
                ]
            ),
            Channel::new(39)
        );

        Bluetooth {
            phy,
        }
    }

    pub fn on_radio_interrupt(&mut self) {
        // self.phy.dump_registers();
        self.phy.on_radio_interrupt();
    }
}
