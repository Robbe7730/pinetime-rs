use rubble::config::Config;
use rubble_nrf5x::timer::BleTimer;
use rubble_nrf5x::radio::BleRadio;
use rubble::l2cap::BleChannelMap;
use rubble::security::NoSecurity;
use rubble::link::queue::SimpleQueue;

use super::attribute_provider::BluetoothAttributeProvider;

use crate::pinetimers::BluetoothTimer;

pub struct BluetoothConfig {}

impl Config for BluetoothConfig {
    type Timer = BleTimer<BluetoothTimer>;
    type Transmitter = BleRadio;
    type ChannelMapper = BleChannelMap<BluetoothAttributeProvider, NoSecurity>;
    type PacketQueue = &'static mut SimpleQueue;
}
