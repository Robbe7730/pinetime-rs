MEMORY
{
 /* ---- INTERNAL FLASH ---- */
 /* BOOTLOADER : ORIGIN = 0x00000000, LENGTH = 28K */
 /* REBOOTLOG : ORIGIN =  0x00007000, LENGTH = 4K */
    HEADER : ORIGIN = 0x00008000, LENGTH = 32
    FLASH : ORIGIN = 0x00008020, LENGTH = 475104 /* 464K - 32 (HEADER) */
 /* SCRATCH : ORIGIN =    0x0007c000, LENGTH = 4K */

 /* ---- EXTERNAL FLASH ---- */
 /* BOOTLOADERASSETS : ORIGIN = 0x00000000, LENGTH = 4K */
 /* STANDBY_IMAGE :    ORIGIN = 0x00040000, LENGTH = 464K */
 /* USER_FILESYSTEM :  ORIGIN = 0x000b4000, LENGTH = 3376K */

 /* ---- RAM ---- */
    RAM : ORIGIN = 0x20000000, LENGTH = 64K
}


/* Source: https://github.com/david-boles/pinetime-rust-mcuboot/blob/main/memory.x */
SECTIONS {
    .mcuboot_header : {
      FILL(0xAAAAAAAA)
      . = . + 32;
    } > HEADER
}
