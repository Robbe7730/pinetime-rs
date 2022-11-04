mod channels;
pub mod packets;

use core::fmt::Debug;

use nrf52832_hal::pac::{RADIO, FICR};

use rtt_target::rprintln;
use alloc::vec;
use alloc::vec::Vec;

use crate::drivers::bluetooth::phy::packets::BluetoothPacket;

pub use self::channels::Channel;

#[derive(Debug)]
pub struct InvalidStateError {}

#[derive(Debug, PartialEq)]
pub enum PhyState {
    Disabled,
    RxRU,
    RxIdle,
    Rx,
    TxRU,
    TxIdle,
    Tx,
    TxDisable,
    RxDisable,
}

impl From<u32> for PhyState {
    fn from(number: u32) -> Self {
        match number {
            0 => PhyState::Disabled,
            1 => PhyState::RxRU,
            2 => PhyState::RxIdle,
            3 => PhyState::Rx,
            4 => PhyState::RxDisable,
            9 => PhyState::TxRU,
            10 => PhyState::TxIdle,
            11 => PhyState::Tx,
            12 => PhyState::TxDisable,
            x => panic!("Invalid RADIO state: {}", x)
        }
    }
}

impl PhyState {
    pub fn is_tx(&self) -> bool {
        self == &PhyState::Tx ||
        self == &PhyState::TxRU ||
        self == &PhyState::TxIdle ||
        self == &PhyState::TxDisable
    }

    pub fn is_rx(&self) -> bool {
        self == &PhyState::Rx ||
        self == &PhyState::RxRU ||
        self == &PhyState::RxIdle ||
        self == &PhyState::RxDisable
    }
}

pub struct PhyRadio {
    radio: RADIO,
    _ficr: FICR,
    packet_buffer: &'static mut [u8; 258],
    packet_queue: Vec<(Channel, BluetoothPacket)>, // (channel, packet)
}

impl PhyRadio {
    pub fn new(
        radio: RADIO,
        ficr: FICR,
        packet_buffer: &'static mut  [u8; 258]
    ) -> Self {
        let ret = Self {
            radio,
            _ficr: ficr,
            packet_buffer,
            packet_queue: vec![]
        };

        // Set MODE to 1 Mbit/s BLE
        ret.radio.mode.write(|w| w.mode().ble_1mbit());

        // Set Tx power to +4dB
        ret.radio.txpower.write(|w| w
            .txpower().pos4d_bm()
        );

        // Set PCNF0
        unsafe {
            ret.radio.pcnf0.write(|w| w
                // LENGTH is 8 bits
                .lflen().bits(8)
                // S0 is 1 byte
                .s0len().bit(true)
                // S1 is 0 bytes (unused)
                .s1len().bits(0)
                // Don't include S1 in ram
                .s1incl().automatic()
                // Preable is 8 bit in BLE 1Mbit/s
                .plen()._8bit()
            );
        }

        // Set PCNF1
        unsafe {
            ret.radio.pcnf1.write(|w| w
                // Set max packet length to 255
                .maxlen().bits(255)
                // Set base address length to 3 bytes (total 4 bytes)
                .balen().bits(3)
                // Enable whitening
                // "Whitening shall be applied on the PDU and CRC ... No other
                // parts of the packets are whitened"
                .whiteen().bit(true)
            )
        }

        // Enable CRC
        ret.radio.crccnf.write(|w| w
            .len().three()     // "The CRC is 24 bits in length and" -> 3 bytes
            .skipaddr().skip() // "the value is calculated over all the PDU bits"
                               // -> "skip" the addr field
        );

        // Set CRC polynomial
        unsafe {
            // x^24 + x^10 + x^9 + x^6 + x^4 + x^3 + x^1 + x^0.
            ret.radio.crcpoly.write(|w| w
                .crcpoly().bits(
                    // (1 << 24) | Only use 24 least significant bits
                    (1 << 10) |
                    (1 <<  9) |
                    (1 <<  6) |
                    (1 <<  4) |
                    (1 <<  3) |
                    (1 <<  1) |
                    (1 <<  0) // Hard-wired to be 1, but it's better to set it anyway
                )
            )
        }

        // Set RX/TX address(es)
        // 0 -> Advertising access address
        unsafe {
            let advertising_access_address = 0x8E89BED6;
            ret.radio.base0.write(|w| w
                .base0().bits(advertising_access_address << 8)
            );
            ret.radio.prefix0.write(|w| w
                .ap0().bits((advertising_access_address >> 24) as u8)
            );
        }

        // Set TX Address to 0 (advertising, set above)
        unsafe {
            ret.radio.txaddress.write(|w| w.txaddress().bits(0));
        }

        // Set RX Address to 0 (advertising, set above)
        ret.radio.rxaddresses.write(|w| w
            .addr0().enabled()
        );

        // Set TIFS to 150µs
        unsafe {
            ret.radio.tifs.write(|w| w
                .tifs().bits(150u8)
            )
        }

        // Set packet pointer
        unsafe {
            let packetptr = ret.packet_buffer.as_mut_ptr() as u32;
            ret.radio.packetptr.write(|w| w
                .packetptr().bits(packetptr)
            );
        }

        ret.enable_interrupts();

        ret
    }

    pub fn set_channel(
        &self,
        channel: Channel,
        crc_iv: Option<u32>,
        access_address: Option<u32>
    ) {
        if crc_iv.is_none() && !channel.is_advertising() {
            panic!("No CRC IV given in data channel")
        }

        if access_address.is_none() && !channel.is_advertising() {
            panic!("No Access Address given in data channel")
        }

        let crc_iv_value = crc_iv.unwrap_or(0x555555);

        unsafe {
            // Set the whitening IV
            self.radio.datawhiteiv.write(|w| w
                .datawhiteiv().bits(channel.whitening_iv())
            )
        }

        unsafe {
            self.radio.crcinit.write(|w| w
                .crcinit().bits(crc_iv_value)
            )
        }

        unsafe {
            // Set frequency offset of 2400 (default map)
            self.radio.frequency.write(|w| w
                .frequency().bits((channel.frequency - 2400) as u8)
                .map().bit(false)
            )
        }

        if let Some(aa) = access_address {
            unsafe {
                // Set base address (low 8 bytes)
                self.radio.base1.write(|w| w
                    .base1().bits(aa << 8)
                );

                // Set prefix (high byte)
                self.radio.prefix0.write(|w| w
                    .ap1().bits((aa >> 24) as u8)
                );
            }
        }
    }

    // Start transition to TxIdle state
    // NOTE: not using Tx state as this requires more setup
    pub fn tx_enable(&self) {
        self.radio.shorts.write(|w| w
            .ready_start().bit(false)
            .end_start().bit(false)
            .end_disable().bit(false)
            .disabled_rxen().bit(false)
            .disabled_txen().bit(true)
        );

        let curr_state = self.get_state();
        if curr_state == PhyState::Rx ||
            curr_state == PhyState::RxIdle ||
            curr_state == PhyState::RxRU
        {
            unsafe {
                self.radio.tasks_disable.write(|w| w
                    .bits(1)
                )
            }
        } else if curr_state == PhyState::Disabled {
            unsafe {
                self.radio.tasks_txen.write(|w| w
                    .bits(1)
                );
            }
        }
    }

    // Start transition to Rx state
    // NOTE: interrupts transmission if one is underway
    pub fn rx_enable(&self) {
        self.radio.shorts.write(|w| w
            .ready_start().bit(true)
            .end_start().bit(true)
            .end_disable().bit(false)
            .disabled_txen().bit(false)
            .disabled_rxen().bit(true)
        );

        let curr_state = self.get_state();
        if curr_state == PhyState::Tx ||
            curr_state == PhyState::TxIdle ||
            curr_state == PhyState::TxRU
        {
            unsafe {
                self.radio.tasks_disable.write(|w| w
                    .bits(1)
                )
            }
        } else if curr_state == PhyState::Disabled {
            unsafe {
                self.radio.tasks_txen.write(|w| w
                    .bits(1)
                );
            }
        }
    }

    pub fn get_state(&self) -> PhyState {
        return self.radio.state.read().bits().into();
    }

    fn set_interrupts(&self, value: bool) {
        self.radio.intenset.write(|w| w
            .ready().bit(value)
            .address().bit(value)
            .payload().bit(value)
            .end().bit(value)
            .disabled().bit(value)
            .devmatch().bit(value)
            .devmiss().bit(value)
            .rssiend().bit(value)
            .bcmatch().bit(value)
            .crcok().bit(value)
            .crcerror().bit(value)
        )
    }

    fn enable_interrupts(&self) {
        self.set_interrupts(true);
    }

    fn disable_interrupts(&self) {
        self.set_interrupts(false);
    }

    pub fn queue_packet(&mut self, packet: BluetoothPacket, channel: Channel) {
        self.packet_queue.push((channel, packet));
        self.tx_enable();
    }

    // Sends the next packet if there is one in the queue
    // If the queue is empty, transition to Rx
    fn process_packet_queue(&mut self) {
        if self.get_state() == PhyState::TxIdle {
            if let Some((channel, packet_to_send)) = self.packet_queue.pop() {
                // Tune the device
                self.set_channel(channel, None, None);

                // Copy the packet into the buffer
                let packet_bytes = packet_to_send.to_bytes();
                // rprintln!("{:?}", packet_bytes);
                self.packet_buffer[0..packet_bytes.len()]
                    .copy_from_slice(packet_bytes.as_slice());
                // Start transmitting
                unsafe {
                    self.radio.tasks_start.write(|w| w
                        .bits(1)
                    );
                }
            } else {
                self.rx_enable();
            }
        } else if self.packet_queue.is_empty() {
            self.rx_enable();
        }
    }

    pub fn on_radio_interrupt(&mut self) {
        self.disable_interrupts();
        if self.radio.events_ready.read().bits() != 0 {
            self.radio.events_ready.reset();
            rprintln!("READY");
            self.process_packet_queue();
        } else if self.radio.events_address.read().bits() != 0 {
            self.radio.events_address.reset();
            rprintln!("ADDRESS");
        } else if self.radio.events_payload.read().bits() != 0 {
            self.radio.events_payload.reset();
            rprintln!("PAYLOAD");
        } else if self.radio.events_end.read().bits() != 0 {
            self.radio.events_end.reset();
            rprintln!("END");
            self.process_packet_queue();
            if self.get_state().is_rx() {
                rprintln!("{:?}", BluetoothPacket::from_advertising_primary(
                    self.packet_buffer
                ));
            }
        } else if self.radio.events_disabled.read().bits() != 0 {
            self.radio.events_disabled.reset();
            rprintln!("DISABLED");
        } else if self.radio.events_devmatch.read().bits() != 0 {
            self.radio.events_devmatch.reset();
            rprintln!("DEVMATCH");
        } else if self.radio.events_devmiss.read().bits() != 0 {
            self.radio.events_devmiss.reset();
            rprintln!("DEVMISS");
        } else if self.radio.events_rssiend.read().bits() != 0 {
            self.radio.events_rssiend.reset();
            rprintln!("RSSIEND");
        } else if self.radio.events_bcmatch.read().bits() != 0 {
            self.radio.events_bcmatch.reset();
            rprintln!("BCMATCH");
        } else if self.radio.events_crcok.read().bits() != 0 {
            self.radio.events_crcok.reset();
            rprintln!("CRCOK");
        } else if self.radio.events_crcerror.read().bits() != 0 {
            self.radio.events_crcerror.reset();
            rprintln!("CRCERROR");
        }
        self.enable_interrupts();
    }

    pub fn dump_registers(&self) {
        rprintln!("---- Radio registers ----");
        rprintln!("SHORTS: {:09b}", self.radio.shorts.read().bits());
        rprintln!("INTENSET: {:014b}", self.radio.intenset.read().bits());
        rprintln!("INTENCLR: {:014b}", self.radio.intenclr.read().bits());
        rprintln!("CRCSTATUS: {:01b}", self.radio.crcstatus.read().bits());
        rprintln!("RXMATCH: {}", self.radio.rxmatch.read().bits());
        rprintln!("RXCRC: {:06x}", self.radio.rxcrc.read().bits());
        rprintln!("DAI: {}", self.radio.dai.read().bits());
        rprintln!("PACKETPTR: {:08x}", self.radio.packetptr.read().bits());
        rprintln!("FREQUENCY: {}", self.radio.frequency.read().bits());
        rprintln!("TXPOWER: {}", self.radio.txpower.read().bits() as i8);
        rprintln!("MODE: {}", self.radio.mode.read().bits());
        rprintln!("PCNF0: {:025b}", self.radio.pcnf0.read().bits());
        rprintln!("PCNF1: {:026b}", self.radio.pcnf1.read().bits());
        rprintln!("BASE0: {:08x}", self.radio.base0.read().bits());
        rprintln!("BASE1: {:08x}", self.radio.base1.read().bits());
        rprintln!("PREFIX0: {:08x}", self.radio.prefix0.read().bits());
        rprintln!("PREFIX1: {:08x}", self.radio.prefix1.read().bits());
        rprintln!("TXADDRESS: {}", self.radio.txaddress.read().bits());
        rprintln!("RXADDRESSES: {:08b}", self.radio.rxaddresses.read().bits());
        rprintln!("CRCCNF: {:02x}", self.radio.crccnf.read().bits());
        rprintln!("CRCPOLY: {:024b}", self.radio.crcpoly.read().bits());
        rprintln!("CRCINIT: {:024b}", self.radio.crcinit.read().bits());
        rprintln!("TIFS: {}", self.radio.tifs.read().bits());
        rprintln!("RSSISAMPLE: {}", self.radio.rssisample.read().bits());
        rprintln!("STATE: {}", self.radio.state.read().bits());
        rprintln!("DATAWHITEIV: {:07b}", self.radio.datawhiteiv.read().bits());
        rprintln!("BCC: {}", self.radio.bcc.read().bits());
        rprintln!("DAB[0]: {:08x}", self.radio.dab[0].read().bits());
        rprintln!("DAB[1]: {:08x}", self.radio.dab[1].read().bits());
        rprintln!("DAB[2]: {:08x}", self.radio.dab[2].read().bits());
        rprintln!("DAB[3]: {:08x}", self.radio.dab[3].read().bits());
        rprintln!("DAB[4]: {:08x}", self.radio.dab[4].read().bits());
        rprintln!("DAB[5]: {:08x}", self.radio.dab[5].read().bits());
        rprintln!("DAB[6]: {:08x}", self.radio.dab[6].read().bits());
        rprintln!("DAB[7]: {:08x}", self.radio.dab[7].read().bits());
        rprintln!("DAP[0]: {:04x}", self.radio.dap[0].read().bits());
        rprintln!("DAP[1]: {:04x}", self.radio.dap[1].read().bits());
        rprintln!("DAP[2]: {:04x}", self.radio.dap[2].read().bits());
        rprintln!("DAP[3]: {:04x}", self.radio.dap[3].read().bits());
        rprintln!("DAP[4]: {:04x}", self.radio.dap[4].read().bits());
        rprintln!("DAP[5]: {:04x}", self.radio.dap[5].read().bits());
        rprintln!("DAP[6]: {:04x}", self.radio.dap[6].read().bits());
        rprintln!("DAP[7]: {:04x}", self.radio.dap[7].read().bits());
        rprintln!("DACNF: {:016b}", self.radio.dacnf.read().bits());
        rprintln!("MODECNF0: {:010b}", self.radio.modecnf0.read().bits());
        rprintln!("POWER: {:01b}", self.radio.power.read().bits());
    }
}
