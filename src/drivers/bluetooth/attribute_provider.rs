use rubble::att::{AttributeProvider, HandleRange, Attribute, Handle, AttUuid};
use rubble::uuid::Uuid16;
use rubble::Error;

use crate::drivers::battery::{Battery, BatteryState};

use alloc::vec::Vec;
use alloc::vec;

use core::ops::BitOr;

#[derive(Debug)]
pub enum ServiceUUID {
    Battery,
}

impl ServiceUUID {
    pub fn data(&self) -> Vec<u8> {
        match self {
            ServiceUUID::Battery => vec![0x0F, 0x18],
        }
    }
}

#[derive(Debug)]
pub enum CharacteristicUUID {
    BatteryLevel,
}

impl From<&CharacteristicUUID> for u16 {
    fn from(uuid: &CharacteristicUUID) -> u16 {
        match uuid {
            CharacteristicUUID::BatteryLevel => 0x2a19,
        }
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
            BluetoothAttribute::CharacteristicValue(uuid, _) => Uuid16(uuid.into()).into(),
        }
    }
}

impl BluetoothAttribute {
    pub fn data(&self, handle: u16) -> Vec<u8>{
        match self {
            BluetoothAttribute::PrimaryService(uuid) => uuid.data(),
            BluetoothAttribute::SecondaryService(uuid) => uuid.data(),
            BluetoothAttribute::Characteristic(prop, uuid) => {
                let next_handle: u16 = handle + 1;
                let uuid_value: u16 = uuid.into();
                vec![
                    prop.into(),
                    (next_handle & 0xff) as u8,
                    ((next_handle >> 8) & 0xff) as u8,
                    (uuid_value & 0xff) as u8,
                    ((uuid_value >> 8) & 0xff) as u8,
                ]
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

    pub fn update_data(&mut self, battery: &mut Battery) {
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
                            )
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
}
