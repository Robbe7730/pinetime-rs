use rtt_target::rprintln;

use super::phy::packets::BluetoothPacket;

trait PacketListener {
    fn on_packet(packet: &BluetoothPacket) {
        rprintln!("{:?}", packet)
    }
}
