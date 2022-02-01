mod poes;
mod main;

pub use poes::ScreenPoes;
pub use main::ScreenMain;

use crate::drivers::touchpanel::TouchPanelEventHandler;
use crate::devicestate::DeviceState;

use core::fmt::Debug;

use alloc::sync::Arc;

pub trait Screen<D> : Send + Debug {
    fn new() -> Self where Self: Sized;
    // I'm using a get_event_handler function because we can't upcast Screen
    // (with Screen : TouchPanelEventHandler) to TouchPanelEventHandler,
    // because we don't know the type (and size) of the current screen...
    fn get_event_handler(&self) -> Arc<dyn TouchPanelEventHandler>;
    fn draw(&self, display: &mut D, devicestate: &DeviceState);
}
