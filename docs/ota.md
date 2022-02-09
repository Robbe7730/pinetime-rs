# OTA Update

## Sequence

The client and the watch communicate through 2 characteristics, the control characteristic and the data characteristic. The client can write to the control and data characteristic directly and the watch can send notifications from the control characteristic.

The client can send the following commands to the control characteristic:

- `[01]`: Request a device reboot
- `[02 00 SS SS SS SS]`: Request firmware update of size S (unsigned, 32 bit integer)
- `[02 ff]`: Abort firmware update

The client can send data to the data characteristic in the following format:

- `[02 NN DD DD DD DD DD DD DD DD]`: Firmware update data packet N (unsigned 8 bit integer, starting at 00, wrapping from ff to 00) with 8 bytes of data D.

If the size of the firmware is not divisible into 8 byte packets, the remaining bytes in the last packet will be ignored.

The watch can send the notifications, each start with one byte indicating the status of the controller (`00`: Normal operation, `01`: Reboot, `02`: Firmware Update), the next byte indicates the status as a **signed** 8 bit integer, where negative values indicate failure and positive values indicate success. The meaning of these status codes depends on the subject, except for `00`, which always indicates success:

- For Normal operation (`00`) no other status codes are used.
- For Reboot (`01`) a status code of -1 (`ff`) indicates that the reboot request was rejected.
- For Firmware Update (`02`) values -1 (`ff`) to -64 (`c0`) indicate a failure that cannot be recovered, while values -65 (`bf`) to -128 (`80`) indicate a failure that can be recovered from. The following status codes exist:
  - `ff`: General, unspecified failure
  - `fe`: Firmware size too large
  - `bf`: General, unspecified failure
  - `be`: Packet number out of order (check the value of the control characteristic for the last succesfully received packet)
  - `bd`: Processing the packet failed
  - `00`: Firmware update completed
  - `01`: Firmware update initiated
  - `02`: Packet received (check the value of the control characteristic for the last succesfully received packet)

The value of the control characteristic also contains data useful for the client, the first byte contains the status of the controller (see above), depending on this, the remainder of the packet can have different meanings

- For Normal operation (`00`) no further data is sent.
- For Reboot (`01`) no further data is sent.
- For Firmware Update (`02`) the remaining packet consists of 1 byte indicating the number of the last successfully received packet and 4 bytes containing the number of bytes successfully received so far.

### Example: Normal update

- WC = Watch (control characteristic)
- WD = Watch (data characteristic)
- C = Client

All lowercase and numeric values are hexadecimal, upper case values are variables.

- C -> WC: `[02 00 SS SS SS SS]` (Request firmware update of size S
  bytes)
- WC -> C: `[02 01]` (Firmware update initiated)
- C -> WD: `[02 NN DD DD DD DD DD DD DD DD]` (Packet number N with 8 bytes
  data D)
- WC -> C: `[02 02]` (Successfully received and processed data)
- *Previous 2 steps repeat until S bytes have been received*
- WC -> C: `[02 00]` (Firmware update queued, waiting for reboot)
- (optional) C -> WC: `[01]` (Request reboot)
- (optional) WC -> C: `[01 00]` (Reboot accepted)


