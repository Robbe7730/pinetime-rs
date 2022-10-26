# pinetime-rs

Pronounced Pine-Timers, my operating system for the [PineTime](https://wiki.pine64.org/wiki/PineTime) using [rtic](https://rtic.rs/).

## Roadmap

- [x] rtic
    - [x] Hardware tasks
    - [x] Software tasks
    - [x] GPIOTE tasks
- [x] ST7789 display
    - [x] Driver
    - [x] [embedded_graphics](https://github.com/embedded-graphics/embedded-graphics) interface
- [x] CST816S Touch controller
- [x] XT25F32B-S 4MiB external flash
    - [ ] Buffered read/write (page-level to allow page erase)
    - [ ] Index trait interface?
- [x] Real-Time Clock
- [ ] Bluetooth
    - [x] Driver
    - [x] Read battery percentage
    - [x] Read/write datetime
    - [ ] OTA firmware update
        - Follow [InfiniTime's DFU protocol](https://github.com/InfiniTimeOrg/InfiniTime/blob/develop/doc/ble.md#firmware-upgrades)?
- [ ] MCUBoot/InfiniTime bootloader support
    - [x] Memory location (0x8000 instead of 0x0000)
    - [x] Watchdog petting
    - [ ] Verifying firmware
- [ ] HRS3300 Heartrate Sensor
- [ ] BMA423 Accelerometer
    - [ ] Step Counter
    - [ ] Activity Recognition: Running, Walking, Still
    - [ ] Tilt-On-Wrist detection
    - [ ] Tap/Double tap interrupt (for disabled touch panel?) 

## Allocations

### GPIOTE Interrupts

1. Push button
2. Touch panel
3. Charging state

### SPI/TWI channels

0. SPIM
1. TWIM

## Setup

To build and flash, simply run `make`. To get the RTT output, use `telnet
localhost 6969`
