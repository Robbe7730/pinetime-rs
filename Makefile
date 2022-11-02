VERSION := $(shell cat Cargo.toml | grep "^version = " | head -n 1 | sed 's/^version = "\(.*\)".*/\1/g')

flash_release: build
	# Create the bin file
	arm-none-eabi-objcopy \
		-R .bss \
		-R .bss.core \
		-R .bss.core.nz \
		-O binary \
		target/thumbv7em-none-eabihf/release/pinetime-rs \
		target/thumbv7em-none-eabihf/release/pinetime-rs.bin
	# Create the image
	imgtool create \
		--align 4 \
		--version $(VERSION) \
		--header-size 32 \
		--slot-size 475136 \
		--pad-header \
		--pad \
		target/thumbv7em-none-eabihf/release/pinetime-rs.bin\
		target/pinetime-rs.img
	# Verify the image
	imgtool verify target/pinetime-rs.img
	# Flash the image
	openocd -c 'source scripts/flash.ocd'

build:
	cargo build --release
