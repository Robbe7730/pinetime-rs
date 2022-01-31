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
- [ ] Real-Time Clock
- [ ] Bluetooth
- [ ] MCUBoot/InfiniTime bootloader support
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
