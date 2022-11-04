use alloc::fmt::Debug;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;

// https://web.archive.org/web/20200722194743/https://www.bluetooth.com/specifications/assigned-numbers/generic-access-profile/
// or https://web.archive.org/web/20210726153139/https://btprodspecificationrefs.blob.core.windows.net/assigned-numbers/Assigned%20Number%20Types/Generic%20Access%20Profile.pdf
// The original no longer exists >:(
#[derive(Debug)]
pub enum AdvData {
    Flags(u8),
    ShortenedLocalName(String),
    CompleteLocalName(String),
    TxPowerLevel(i8),

    Unknown(Vec<u8>)
}

impl AdvData {
    fn data_from_pdu(data: &[u8]) -> Vec<AdvData> {
        let mut i = 0;

        let mut ret = Vec::new();

        while i < data.len() {
            let len: usize = data[i].into();
            i += 1;

            if len == 0 {
                break
            }

            if (i + len) > data.len() {
                ret.push(
                    AdvData::Unknown(Vec::from(&data[i..]))
                )
            } else {
                ret.push(
                    AdvData::from(&data[i..i+len])
                )
            }

            i += len;
        }

        ret
    }

    fn to_bytes(self) -> Vec<u8> {
        match self {
            Self::Flags(f) => vec![1, f],
            Self::ShortenedLocalName(n) | Self::CompleteLocalName(n) => {
                let mut ret = vec![u8::try_from(n.len()).unwrap()];
                ret.append(&mut n.into_bytes());
                ret
            }
            Self::TxPowerLevel(l) => vec![1, l as u8],
            Self::Unknown(data) => data,
        }
    }
}

impl From<&[u8]> for AdvData {
    fn from(data: &[u8]) -> Self {
        match data[0] {
            1 => AdvData::Flags(data[1]),
            8 => AdvData::ShortenedLocalName(
                String::from_utf8_lossy(&data[1..]).into()
            ),
            9 => AdvData::CompleteLocalName(
                String::from_utf8_lossy(&data[1..]).into()
            ),
            10 => AdvData::TxPowerLevel(data[1] as i8),
            _ => AdvData::Unknown(Vec::from(data))
        }
    }
}

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

    pub fn to_bytes(self) -> [u8; 6] {
        match self {
            Self::Public(x) | Self::Random(x) => x
        }
    }

    pub fn is_random(&self) -> bool {
        match self {
            Self::Public(_) => false,
            Self::Random(_) => true,
        }
    }
}

#[derive(Debug)]
pub enum BluetoothPacket {
    // PRIMARY ADVERTISING
    AdvInd(BluetoothAddress, Vec<AdvData>), // AdvA, AdvData
    AdvDirectInd(BluetoothAddress, BluetoothAddress), // AdvA,TargetA
    AdvNonconnInd(BluetoothAddress, Vec<AdvData>), // AdvA, AdvData
    ScanReq(BluetoothAddress, BluetoothAddress), // ScanA, AdvA
    ScanRsp(BluetoothAddress, ()), // AdvA, ScanRspData (TODO)
    ConnectInd(BluetoothAddress, BluetoothAddress, [u8; 22]), // InitA, AdvA, LLData
    AdvScanInd(BluetoothAddress, Vec<AdvData>), // AdvA, AdvData
    AdvExtInd(()), // TODO

    Unkown(u8, Vec<u8>),
}

impl BluetoothPacket {
    pub fn from_advertising_primary(data: &[u8]) -> Result<Self, ()> {
        let pdu_type = data[0] & 0x0f;
        let _chsel = (data[0] & 0b00100000) != 0;
        let txadd = (data[0] & 0b01000000) != 0;
        let rxadd = (data[0] & 0b10000000) != 0;

        let length = data[1] as usize;
        let pdu: &[u8] = &data[2..(length+2)];

        Ok(match pdu_type {
            0b0000 => Self::AdvInd(
                BluetoothAddress::new(&pdu[0..6], txadd)?,
                AdvData::data_from_pdu(&pdu[6..])
            ),
            0b0001 => Self::AdvDirectInd(
                BluetoothAddress::new(&pdu[0..6], txadd)?,
                BluetoothAddress::new(&pdu[6..12], rxadd)?
            ),
            0b0010 => Self::AdvNonconnInd(
                BluetoothAddress::new(&pdu[0..6], txadd)?,
                AdvData::data_from_pdu(&pdu[6..])
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
                AdvData::data_from_pdu(&pdu[6..])
            ),
            0b0111 => Self::AdvExtInd(()),
            x => BluetoothPacket::Unkown(x, Vec::from(pdu))
        })

    }


    pub fn to_bytes(self) -> Vec<u8> {
        match self {
            Self::AdvInd(addr, data) => {
                let mut length = 6;
                let mut ret = vec![0, 0];

                if addr.is_random() {
                    ret[0] = 0b01000000;
                }

                ret.extend_from_slice(&addr.to_bytes());

                for advdata in data {
                    let mut advdata_bytes = advdata.to_bytes();
                    length += u8::try_from(advdata_bytes.len()).unwrap();
                    ret.append(&mut advdata_bytes);
                }

                ret[1] = length;

                return ret;
            },
            _ => todo!()
        }
    }
}
