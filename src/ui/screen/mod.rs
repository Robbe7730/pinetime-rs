mod main;

pub use main::ScreenMain;

use crate::drivers::bluetooth::Bluetooth;
use crate::drivers::touchpanel::TouchPanelEventHandler;
use crate::drivers::clock::Clock;
use crate::drivers::mcuboot::MCUBoot;

use crate::pinetimers::ConnectedRtc;

use core::fmt::Debug;

use alloc::sync::Arc;

pub trait Screen<D> : Send + Debug {
    fn new() -> Self where Self: Sized;
    // I'm using a get_event_handler function because we can't upcast Screen
    // (with Screen : TouchPanelEventHandler) to TouchPanelEventHandler,
    // because we don't know the type (and size) of the current screen...
    fn get_event_handler(&self) -> Arc<dyn TouchPanelEventHandler>;
    fn draw_init(&mut self, display: &mut D, clock: &Clock<ConnectedRtc>, mcuboot: &MCUBoot, bluetooth: &Bluetooth);
    fn draw_update(&mut self, display: &mut D, clock: &Clock<ConnectedRtc>, mcuboot: &MCUBoot, bluetooth: &Bluetooth);
}
