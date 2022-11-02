#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

mod drivers;
mod ui;
mod pinetimers;

extern crate alloc;

// This module is basically a pass-through for pinetimers::tasks_impl, which 
// makes it possible to have separate files for tasks.
#[rtic::app(device = nrf52832_hal::pac, dispatchers = [SWI0_EGU0, SWI1_EGU1])]
mod tasks {
    use crate::drivers::timer::MonoTimer;
    use crate::drivers::display::Display;
    use crate::drivers::touchpanel::TouchPanel;
    use crate::drivers::flash::{InternalFlash, ExternalFlash};
    use crate::drivers::bluetooth::Bluetooth;
    use crate::drivers::battery::Battery;
    use crate::drivers::clock::Clock;
    use crate::drivers::mcuboot::MCUBoot;

    use crate::ui::screen::Screen;

    use crate::pinetimers::{ConnectedSpim, PixelType, ConnectedRtc};

    use nrf52832_hal::pac::TIMER0;
    use nrf52832_hal::gpiote::Gpiote;
    use nrf52832_hal::spim::Spim;
    use nrf52832_hal::wdt::WatchdogHandle;
    use nrf52832_hal::wdt::handles::HdlN;

    use rubble::link::MIN_PDU_BUF;
    use rubble_nrf5x::radio::PacketBuffer;
    use rubble::link::queue::SimpleQueue;

    use chrono::NaiveDateTime;

    use alloc::boxed::Box;

    use spin::Mutex;

    #[monotonic(binds = TIMER0, default = true)]
    type Mono0 = MonoTimer<TIMER0>;

    #[shared]
    struct Shared {
        gpiote: Gpiote,
        watchdog_handles: [WatchdogHandle<HdlN> ; 1],

        display: Display<PixelType, ConnectedSpim>,
        touchpanel: TouchPanel,
        internal_flash: InternalFlash,
        external_flash: ExternalFlash,
        bluetooth: Bluetooth,
        battery: Battery,
        clock: Clock<ConnectedRtc>,
        mcuboot: MCUBoot,

        current_screen: Box<dyn Screen<Display<PixelType, ConnectedSpim>>>,
    }

    #[local]
    struct Local {}

    // I'm using a separate struct and into() to allow the init function
    // to be in a separate crate as Shared and Local cannot be made pub
    impl From<crate::pinetimers::tasks_impl::init::Shared> for Shared {
        fn from(init_shared: crate::pinetimers::tasks_impl::init::Shared) -> Shared {
            Shared {
                gpiote: init_shared.gpiote,
                watchdog_handles: init_shared.watchdog_handles,

                display: init_shared.display,
                touchpanel: init_shared.touchpanel,
                internal_flash: init_shared.internal_flash,
                external_flash: init_shared.external_flash,
                bluetooth: init_shared.bluetooth,
                battery: init_shared.battery,
                clock: init_shared.clock,
                mcuboot: init_shared.mcuboot,

                current_screen: init_shared.current_screen,
            }
        }
    }

    impl From<crate::pinetimers::tasks_impl::init::Local> for Local {
        fn from(_init_local: crate::pinetimers::tasks_impl::init::Local) -> Local {
            Local {}
        }
    }

    // Allocate here to make them 'static
    #[init(local = [
            spi_lock: Mutex<Option<Spim<crate::pinetimers::ConnectedSpim>>> = Mutex::new(None),
            ble_tx_buf: PacketBuffer = [0; MIN_PDU_BUF],
            ble_rx_buf: PacketBuffer = [0; MIN_PDU_BUF],
            ble_tx_queue: SimpleQueue = SimpleQueue::new(),
            ble_rx_queue: SimpleQueue = SimpleQueue::new(),
    ])]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let (shared, local, mono) = crate::pinetimers::tasks_impl::init(ctx);

        (shared.into(), local.into(), mono)
    }

    #[idle]
    fn idle(ctx: idle::Context) -> ! {
        crate::pinetimers::tasks_impl::idle(ctx)
    }

    #[task(shared = [display])]
    fn display_init(ctx: display_init::Context) {
        crate::pinetimers::tasks_impl::display_init(ctx)
    }

    #[task(binds = GPIOTE, shared = [gpiote, touchpanel, current_screen])]
    fn gpiote_interrupt(ctx: gpiote_interrupt::Context) {
        crate::pinetimers::tasks_impl::gpiote_interrupt(ctx)
    }

    #[task(shared = [clock])]
    fn periodic_update_device_state(ctx: periodic_update_device_state::Context) {
        crate::pinetimers::tasks_impl::periodic_update_device_state(ctx)
    }

    #[task(shared = [display, current_screen, clock, mcuboot])]
    fn redraw_screen(ctx: redraw_screen::Context) {
        crate::pinetimers::tasks_impl::redraw_screen(ctx)
    }

    #[task(shared = [display, current_screen, clock, mcuboot])]
    fn init_screen(ctx: init_screen::Context) {
        crate::pinetimers::tasks_impl::init_screen(ctx)
    }

    #[task(shared = [external_flash, internal_flash, mcuboot])]
    fn self_test(ctx: self_test::Context) {
        crate::pinetimers::tasks_impl::self_test(ctx)
    }

    #[task(shared = [current_screen])]
    fn transition(ctx: transition::Context, new_screen: Box<dyn Screen<Display<PixelType, ConnectedSpim>>>) {
        crate::pinetimers::tasks_impl::transition(ctx, new_screen)
    }

    #[task(binds = RADIO, shared = [bluetooth], priority = 3)]
    fn ble_radio(ctx: ble_radio::Context) {
        crate::pinetimers::tasks_impl::ble_radio(ctx)
    }

    #[task(shared = [bluetooth], priority = 2)]
    fn ble_worker(ctx: ble_worker::Context) {
        crate::pinetimers::tasks_impl::ble_worker(ctx)
    }

    #[task(binds = TIMER2, shared = [bluetooth], priority = 3)]
    fn ble_timer(ctx: ble_timer::Context) {
        crate::pinetimers::tasks_impl::ble_timer(ctx)
    }

    #[task(shared = [bluetooth, battery, clock, mcuboot])]
    fn ble_update(ctx: ble_update::Context) {
        crate::pinetimers::tasks_impl::ble_update(ctx)
    }

    #[task(shared = [clock])]
    fn set_time(ctx: set_time::Context, time: NaiveDateTime) {
        crate::pinetimers::tasks_impl::set_time(ctx, time);
    }

    #[task(shared = [watchdog_handles])]
    fn pet_watchdog(ctx: pet_watchdog::Context) {
        crate::pinetimers::tasks_impl::pet_watchdog(ctx);
    }

    #[task(shared = [mcuboot])]
    fn validate(ctx: validate::Context) {
        crate::pinetimers::tasks_impl::validate(ctx);
    }

    #[task]
    fn reboot(ctx: reboot::Context) {
        crate::pinetimers::tasks_impl::reboot(ctx);
    }
}

use rtt_target::rprintln;

use core::panic::PanicInfo;

use linked_list_allocator::LockedHeap;

use alloc::alloc::Layout;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("----- PANIC -----");
    rprintln!("{:#?}", info);
    loop {
        cortex_m::asm::bkpt();
    }
}

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn on_oom(layout: Layout) -> ! {
    rprintln!("----- OOM -----");
    rprintln!("{:#?}", layout);
    loop {
        cortex_m::asm::bkpt();
    }
}
