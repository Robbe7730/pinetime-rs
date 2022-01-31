mod dummy1;
mod dummy2;

pub use dummy1::ScreenDummy1;
pub use dummy2::ScreenDummy2;

use crate::drivers::touchpanel::TouchPanelEventHandler;
use crate::drivers::display::{Display, DisplaySupported};

use core::fmt::Debug;

use alloc::sync::Arc;

pub trait Screen<COLOR> : Send + Debug
where
    Display<COLOR>: DisplaySupported<COLOR>
{
    fn new() -> Self where Self: Sized;
    fn get_event_handler(&self) -> Arc<dyn TouchPanelEventHandler>;
    fn draw(&self, display: &mut Display<COLOR>);
}
