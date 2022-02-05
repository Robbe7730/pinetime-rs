BLE has different services, every service has charactersitics and every
charactersitic has values. All these are expressed using Attributes,

From bt spec:
- An attribute is a discrete value that has the following three properties
associated with it:
     (1) an attribute type, defined by a UUID, (found in 16-bit UUID Numbers Document)
     (2) an attribute handle
     (3) a set of permissions
- A service is defined by uuid 0x2800 (primary) or 0x2801 (secondary) and
  followed by include (0x2802) or characterstic (0x2803) definitions, in
  order of the handle.
     - (I don't use includes)
     - A characteristic definition ends at the start or the next
       characteristic declaration or service declaration or after the
       maximum Attribute Handle (0xffff).
     - Immediately after a characteristic declaration should be a
       characteristic value declaration, after that any number of
       characteristic descriptors.
- Characteristic definition (data):
     - 1 byte: properties (bitvector)
         - 0x01: Broadcast
         - 0x02: Read
         - 0x04: Write without response
         - 0x08: Write
         - 0x10: Notify
         - 0x20: Indicate
         - 0x40: Authenticated Signed Writes
         - 0x80: Extended Properties
     - 2 bytes: handle of the attribute that contains its value
     - 2/16 bytes: UUID of the value
