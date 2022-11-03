pub mod phy;
mod link;

use nrf52832_hal::pac::{RADIO, FICR};

use self::phy::{PhyRadio, Channel};

pub struct Bluetooth {
    pub phy: PhyRadio,
}

impl Bluetooth {
    pub fn new(
        radio: RADIO,
        ficr: FICR,
        packet_buffer: &'static mut [u8; 258]
    ) -> Self {
        let phy = PhyRadio::new(radio, ficr, packet_buffer);

        phy.set_channel(Channel::new(39), None, None);
        phy.rx_enable().unwrap();

        Bluetooth {
            phy,
        }
    }

    pub fn on_radio_interrupt(&mut self) {
        // self.phy.dump_registers();
        self.phy.on_radio_interrupt();
    }
}
