use alloc::fmt::Debug;

use alloc::vec::Vec;

pub enum BluetoothAddress {
    Random([u8; 6]),
    Public([u8; 6]),
}

impl Debug for BluetoothAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BluetoothAddress::Public(x) => {
                f.write_fmt(format_args!(
                    "Public({:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x})",
                    x[5], x[4], x[3], x[2], x[1], x[0]
                ))
            }
            BluetoothAddress::Random(x) => {
                f.write_fmt(format_args!(
                    "Random({:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x})",
                    x[5], x[4], x[3], x[2], x[1], x[0]
                ))
            }
        }
    }
}

impl BluetoothAddress {
    fn new(data: &[u8], is_random: bool) -> Result<Self, ()> {
        if data.len() < 6 {
            return Err(());
        }

        Ok(if is_random {
            BluetoothAddress::Random(data.try_into().unwrap())
        } else {

            BluetoothAddress::Public(data.try_into().unwrap())
        })
    }
}

#[derive(Debug)]
pub enum BluetoothPacket {
    // PRIMARY ADVERTISING
    AdvInd(BluetoothAddress, ()), // AdvA, AdvData (TODO)
    AdvDirectInd(BluetoothAddress, BluetoothAddress), // AdvA,TargetA
    AdvNonconnInd(BluetoothAddress, ()), // AdvA, AdvData (TODO)
    ScanReq(BluetoothAddress, BluetoothAddress), // ScanA, AdvA
    ScanRsp(BluetoothAddress, ()), // AdvA, ScanRspData (TODO)
    ConnectInd(BluetoothAddress, BluetoothAddress, [u8; 22]), // InitA, AdvA, LLData
    AdvScanInd(BluetoothAddress, ()), // AdvA, AdvData (TODO)
    AdvExtInd(()), // TODO

    Unkown(u8, Vec<u8>),
}

impl BluetoothPacket {
    pub fn from_advertising_primary(data: &[u8]) -> Result<Self, ()> {
        let pdu_type = data[0] & 0x0f;
        let _rfu = (data[0] & 0b00010000) != 0;
        let _chsel = (data[0] & 0b00100000) != 0;
        let txadd = (data[0] & 0b01000000) != 0;
        let rxadd = (data[0] & 0b10000000) != 0;

        let length = data[1] as usize;
        let pdu: &[u8] = &data[2..(length+2)];

        Ok(match pdu_type {
            0b0000 => Self::AdvInd(
                BluetoothAddress::new(&pdu[0..6], txadd)?,
                ()
            ),
            0b0001 => Self::AdvDirectInd(
                BluetoothAddress::new(&pdu[0..6], txadd)?,
                BluetoothAddress::new(&pdu[6..12], rxadd)?
            ),
            0b0010 => Self::AdvNonconnInd(
                BluetoothAddress::new(&pdu[0..6], txadd)?,
                ()
            ),
            0b0011 => Self::ScanReq(
                BluetoothAddress::new(&pdu[0..6], txadd)?,
                BluetoothAddress::new(&pdu[6..12], rxadd)?
            ),
            0b0100 => Self::ScanRsp(
                BluetoothAddress::new(&pdu[0..6], txadd)?,
                ()
            ),
            0b0101 => Self::ConnectInd(
                BluetoothAddress::new(&pdu[0..6], txadd)?,
                BluetoothAddress::new(&pdu[6..12], rxadd)?,
                pdu[12..34].try_into().map_err(|_| ())?
            ),
            0b0110 => Self::AdvScanInd(
                BluetoothAddress::new(&pdu[0..6], txadd)?,
                ()
            ),
            0b0111 => Self::AdvExtInd(()),
            x => BluetoothPacket::Unkown(x, Vec::from(pdu))
        })

    }
}
