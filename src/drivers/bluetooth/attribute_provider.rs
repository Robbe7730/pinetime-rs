use rubble::att::{AttributeProvider, HandleRange, Attribute, Handle, AttUuid, AttributeAccessPermissions};
use rubble::uuid::{Uuid16, Uuid128};
use rubble::bytes::{ByteWriter, ToBytes};
use rubble::Error;

use crate::drivers::battery::{Battery, BatteryState};
use crate::drivers::clock::Clock;

use crate::drivers::mcuboot::MCUBoot;
use crate::pinetimers::ConnectedRtc;

use chrono::{Datelike, Timelike, NaiveDateTime, NaiveDate, NaiveTime};

use alloc::vec::Vec;
use alloc::vec;

use core::ops::BitOr;

#[derive(Debug)]
pub enum ServiceUUID {
    Battery,
    CurrentTime,
    GenericAccess,
    DeviceInformation,
}

impl ServiceUUID {
    pub fn data(&self) -> Vec<u8> {
        match self {
            ServiceUUID::Battery => vec![0x0F, 0x18],
            ServiceUUID::CurrentTime => vec![0x05, 0x18],
            ServiceUUID::GenericAccess => vec![0x00, 0x18],
            ServiceUUID::DeviceInformation => vec![0x0A, 0x18],
        }
    }
}

#[derive(Debug)]
pub enum CharacteristicUUID {
    BatteryLevel,
    DateTime,
    CurrentTime,
    FirmwareRevisionString,
}

impl From<&CharacteristicUUID> for Uuid128 {
    fn from(uuid: &CharacteristicUUID) -> Uuid128 {
        match uuid {
            CharacteristicUUID::BatteryLevel => Uuid16(0x2a19).into(),
            CharacteristicUUID::DateTime => Uuid16(0x2a08).into(),
            CharacteristicUUID::CurrentTime => Uuid16(0x2a2b).into(),
            CharacteristicUUID::FirmwareRevisionString => Uuid16(0x2a26).into(),
        }
    }
}

impl From<&CharacteristicUUID> for AttUuid {
    fn from(uuid: &CharacteristicUUID) -> AttUuid {
        return Uuid128::from(uuid).into();
    }
}

#[derive(Debug)]
pub enum CharacteristicProperty {
    Broadcast,
    Read,
    WriteNoResponse,
    Write,
    Notify,
    Indicate,
    AuthenticatedSignedWrites,
    ExtendedProperties,
    Combination(u8),
}

impl From<&CharacteristicProperty> for u8 {
    fn from(property: &CharacteristicProperty) -> u8 {
        match property {
            CharacteristicProperty::Broadcast => 0x01,
            CharacteristicProperty::Read => 0x02,
            CharacteristicProperty::WriteNoResponse => 0x04,
            CharacteristicProperty::Write => 0x08,
            CharacteristicProperty::Notify => 0x10,
            CharacteristicProperty::Indicate => 0x20,
            CharacteristicProperty::AuthenticatedSignedWrites => 0x40,
            CharacteristicProperty::ExtendedProperties => 0x80,
            CharacteristicProperty::Combination(v) => *v,
        }
    }
}

impl CharacteristicProperty {
    pub fn to_rubble(&self) -> AttributeAccessPermissions {
        if self.includes(CharacteristicProperty::Write) {
            if self.includes(CharacteristicProperty::Read) {
                AttributeAccessPermissions::ReadableAndWriteable
            } else {
                AttributeAccessPermissions::Writeable
            }
        } else {
            AttributeAccessPermissions::Readable
        }
    }

    pub fn includes(&self, other: CharacteristicProperty) -> bool {
        return (u8::from(self) & u8::from(&other)) != 0;
    }
}

impl BitOr for CharacteristicProperty {
    type Output = CharacteristicProperty;

    fn bitor(self, rhs: CharacteristicProperty) -> Self::Output {
        CharacteristicProperty::Combination(
            u8::from(&self) | u8::from(&rhs)
        )
    }
}

#[derive(Debug)]
pub enum BluetoothAttribute {
    PrimaryService(ServiceUUID),
    SecondaryService(ServiceUUID),
    Characteristic(CharacteristicProperty, CharacteristicUUID),
    CharacteristicValue(CharacteristicUUID, Vec<u8>),
}

impl From<&BluetoothAttribute> for AttUuid {
    fn from(bt_att: &BluetoothAttribute) -> AttUuid {
        match bt_att {
            BluetoothAttribute::PrimaryService(_) => Uuid16(0x2800).into(),
            BluetoothAttribute::SecondaryService(_) => Uuid16(0x2801).into(),
            BluetoothAttribute::Characteristic(_, _) => Uuid16(0x2803).into(),
            BluetoothAttribute::CharacteristicValue(uuid, _) => uuid.into(),
        }
    }
}

impl BluetoothAttribute {
    pub fn data(&self, handle: u16) -> Vec<u8>{
        match self {
            BluetoothAttribute::PrimaryService(uuid) => uuid.data(),
            BluetoothAttribute::SecondaryService(uuid) => uuid.data(),
            BluetoothAttribute::Characteristic(prop, uuid) => {
                let properties: u8 = prop.into();
                let next_handle: u16 = handle + 1;

                let mut uuid_buffer = [0; 16];
                let mut uuidwriter = ByteWriter::new(&mut uuid_buffer);
                Uuid128::from(uuid).to_bytes(&mut uuidwriter).unwrap();
                uuid_buffer.reverse();

                let mut bytebuffer = vec![
                    properties,
                    (next_handle & 0xff) as u8,
                    ((next_handle >> 8) & 0xff) as u8,
                ];

                bytebuffer.extend_from_slice(&uuid_buffer);

                return bytebuffer;
            }
            BluetoothAttribute::CharacteristicValue(_, value) => value.clone(),
        }
    }

    pub fn to_rubble(&self, handle: u16) -> Attribute<Vec<u8>> {
        Attribute::new(
            self.into(),
            Handle::from_raw(handle),
            self.data(handle),
        )
    }
}

pub struct BluetoothAttributeProvider {
    attributes: Vec<BluetoothAttribute>,

    // Storing these to make sure they can be returned
    rubble_attributes: Vec<Attribute<Vec<u8>>>,
}

impl BluetoothAttributeProvider {
    pub fn new() -> Self {
        let attributes = vec![
            BluetoothAttribute::PrimaryService(ServiceUUID::Battery),
            BluetoothAttribute::Characteristic(
                CharacteristicProperty::Read,
                CharacteristicUUID::BatteryLevel
            ),
            BluetoothAttribute::CharacteristicValue(
                CharacteristicUUID::BatteryLevel,
                vec![0],
            ),
            BluetoothAttribute::PrimaryService(ServiceUUID::CurrentTime),
            BluetoothAttribute::Characteristic(
                CharacteristicProperty::Read | CharacteristicProperty::Write,
                CharacteristicUUID::DateTime
            ),
            BluetoothAttribute::CharacteristicValue(
                CharacteristicUUID::DateTime,
                vec![0, 0, 0, 0, 0, 0, 0]
            ),
            BluetoothAttribute::Characteristic(
                CharacteristicProperty::Read | CharacteristicProperty::Write,
                CharacteristicUUID::CurrentTime
            ),
            BluetoothAttribute::CharacteristicValue(
                CharacteristicUUID::CurrentTime,
                vec![0, 0, 0, 0, 0, 0, 0, 0, 0]
            ),
            BluetoothAttribute::PrimaryService(ServiceUUID::DeviceInformation),
            BluetoothAttribute::Characteristic(
                CharacteristicProperty::Read,
                CharacteristicUUID::FirmwareRevisionString
            ),
            BluetoothAttribute::CharacteristicValue(
                CharacteristicUUID::FirmwareRevisionString,
                "unknown".as_bytes().to_vec()
            ),
        ];
        let rubble_attributes = Self::rubble_attributes(&attributes);
        Self {
            attributes,
            rubble_attributes,
        }
    }

    fn rubble_attributes(attributes: &Vec<BluetoothAttribute>) -> Vec<Attribute<Vec<u8>>> {
        attributes.iter().enumerate().map(|(i, att)| {
            let handle: u16 = (i + 1).try_into().unwrap();
            att.to_rubble(handle)
        }).collect()
    }

    fn update_rubble_attributes(&mut self) {
        self.rubble_attributes = Self::rubble_attributes(&self.attributes);
    }

    pub fn update_data(
        &mut self,
        battery: &mut Battery,
        clock: &Clock<ConnectedRtc>,
        mcuboot: &MCUBoot
    ) {
        let percentage = match battery.get_state() {
            BatteryState::Charging(x) => x,
            BatteryState::Discharging(x) => x,
            BatteryState::Unknown => 0.0,
        };

        for i in 0..self.attributes.len() {
            match &self.attributes[i] {
                BluetoothAttribute::CharacteristicValue(uuid, _) => {
                    self.attributes[i] = match uuid {
                        CharacteristicUUID::BatteryLevel => 
                            BluetoothAttribute::CharacteristicValue(
                                CharacteristicUUID::BatteryLevel,
                                vec![percentage as u8]
                            ),
                        CharacteristicUUID::DateTime =>
                            BluetoothAttribute::CharacteristicValue(
                                CharacteristicUUID::DateTime,
                                vec![
                                    (clock.datetime.year() & 0xff).try_into().unwrap(),
                                    ((clock.datetime.year() >> 8) & 0xff).try_into().unwrap(),
                                    (clock.datetime.month() & 0xff).try_into().unwrap(),
                                    (clock.datetime.day() & 0xff).try_into().unwrap(),
                                    (clock.datetime.hour() & 0xff).try_into().unwrap(),
                                    (clock.datetime.minute() & 0xff).try_into().unwrap(),
                                    (clock.datetime.second() & 0xff).try_into().unwrap(),
                                ]
                            ),
                        CharacteristicUUID::CurrentTime =>
                            BluetoothAttribute::CharacteristicValue(
                                CharacteristicUUID::CurrentTime,
                                vec![
                                    (clock.datetime.year() & 0xff).try_into().unwrap(),
                                    ((clock.datetime.year() >> 8) & 0xff).try_into().unwrap(),
                                    (clock.datetime.month() & 0xff).try_into().unwrap(),
                                    (clock.datetime.day() & 0xff).try_into().unwrap(),
                                    (clock.datetime.hour() & 0xff).try_into().unwrap(),
                                    (clock.datetime.minute() & 0xff).try_into().unwrap(),
                                    (clock.datetime.second() & 0xff).try_into().unwrap(),
                                    0, // TODO: Day of week
                                    0, // TODO: Fractions of a second
                                    0, // TODO: Reason for update
                                ]
                            ),
                        CharacteristicUUID::FirmwareRevisionString =>
                            BluetoothAttribute::CharacteristicValue(
                                CharacteristicUUID::FirmwareRevisionString,
                                mcuboot.version_string().as_bytes().to_vec()
                            ),
                    };
                },
                _ => {}
            }
        }

        self.update_rubble_attributes();
    }
}

impl AttributeProvider for BluetoothAttributeProvider {
    fn for_attrs_in_range(
        &mut self,
        range: HandleRange,
        mut fun: impl FnMut(&Self, &Attribute<dyn AsRef<[u8]>>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        // Execute the function `fun` for all attributes in the range `range`
        let count = self.attributes.len(); // attributes.len() == rubble_attributes().len()
        let start = usize::from(range.start().as_u16() - 1); // handles start at 1, not 0
        let end = usize::from(range.end().as_u16() - 1);

        let attrs = if start >= count {
            &[]
        } else {
            let end = end.min(count - 1);
            &self.rubble_attributes[start..=end]
        };

        for attr in attrs {
            fun(self, attr)?;
        }
        Ok(())
    }

    fn is_grouping_attr(&self, uuid: AttUuid) -> bool {
        // Check if the uuid is a grouping attribute
        // The example implementation only uses primary service
        uuid == Uuid16(0x2800) || uuid == Uuid16(0x2801)
    }

    fn group_end(&self, handle: Handle) -> Option<&Attribute<(dyn AsRef<[u8]>)>> {
        // Indicate where the group started by `handle` ends (None if no group)
        let start_handle: usize = (handle.as_u16() - 1).into();

        match self.attributes[start_handle] {

            // The service definition ends before the next service declaration
            // or after the maximum Attribute Handle is reached.
            BluetoothAttribute::PrimaryService(_) => {
                let mut handle: usize = start_handle + 1;
                while handle < self.attributes.len() {
                    let curr_attr = &self.attributes[handle];
                    if let BluetoothAttribute::PrimaryService(_) = curr_attr {
                        return Some(&self.rubble_attributes[handle-1]);
                    } else if let BluetoothAttribute::SecondaryService(_) = curr_attr {
                        return Some(&self.rubble_attributes[handle-1]);
                    }

                    handle += 1;
                }

                return Some(&self.rubble_attributes[self.rubble_attributes.len()-1]);
            }
            BluetoothAttribute::SecondaryService(_) => {
                let mut handle: usize = start_handle + 1;
                while handle < self.attributes.len() {
                    let curr_attr = &self.attributes[handle];
                    if let BluetoothAttribute::PrimaryService(_) = curr_attr {
                        return Some(&self.rubble_attributes[handle-1]);
                    } else if let BluetoothAttribute::SecondaryService(_) = curr_attr {
                        return Some(&self.rubble_attributes[handle-1]);
                    }

                    handle += 1;
                }

                return Some(&self.rubble_attributes[self.rubble_attributes.len()-1]);
            }
            // All others are (as far as I know) not groups
            _ => None
        }
    }

    fn attr_access_permissions(&self, handle: Handle) -> AttributeAccessPermissions {
        // Handle 0x0001 is a service, so always Readable
        if handle.as_u16() < 2 {
            return AttributeAccessPermissions::Readable;
        }

        if let BluetoothAttribute::Characteristic(properties, _) = &self.attributes[handle.as_u16() as usize - 2] {
            return properties.to_rubble();
        }

        return AttributeAccessPermissions::Readable;
    }

    fn write_attr(&mut self, handle: Handle, data: &[u8]) -> Result<(), Error> {
        let i: usize = (handle.as_u16() - 1).into();

        match &self.attributes[i] {
            BluetoothAttribute::CharacteristicValue(CharacteristicUUID::DateTime, _) => {
                if data.len() == 7 {
                    self.attributes[i] = BluetoothAttribute::CharacteristicValue(
                        CharacteristicUUID::DateTime,
                        data.to_vec()
                    );
                    crate::tasks::set_time::spawn(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd(
                                i32::from(data[1]) << 8 | i32::from(data[0]),
                                u32::from(data[2]),
                                u32::from(data[3])
                            ),
                            NaiveTime::from_hms(
                                u32::from(data[4]),
                                u32::from(data[5]),
                                u32::from(data[6])
                            )
                        )
                    ).unwrap();
                }
            }
            BluetoothAttribute::CharacteristicValue(CharacteristicUUID::CurrentTime, _) => {
                if data.len() == 10 {
                    self.attributes[i] = BluetoothAttribute::CharacteristicValue(
                        CharacteristicUUID::CurrentTime,
                        data.to_vec()
                    );
                    crate::tasks::set_time::spawn(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd(
                                i32::from(data[1]) << 8 | i32::from(data[0]),
                                u32::from(data[2]),
                                u32::from(data[3])
                            ),
                            NaiveTime::from_hms(
                                u32::from(data[4]),
                                u32::from(data[5]),
                                u32::from(data[6])
                            )
                        )
                    ).unwrap();
                }
            }
            _ => {},
        };

        self.update_rubble_attributes();

        Ok(())
    }
}
