use crate::ui::screen::{Screen, ScreenPoes};
use crate::drivers::touchpanel::{TouchPanelEventHandler, TouchPoint};
use crate::drivers::display::{Display, DisplaySupported};
use crate::devicestate::DeviceState;

use embedded_graphics::prelude::{Point, Drawable, DrawTarget};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::RgbColor;
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::text::{Text, Baseline};

use core::marker::PhantomData;
use core::fmt::Debug;

use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::format;

#[derive(Debug)]
pub struct ScreenMain<COLOR> {
    event_handler: Arc<ScreenMainEventHandler>,
    _marker: PhantomData<COLOR>
}

#[derive(Debug)]
pub struct ScreenMainEventHandler {}

impl TouchPanelEventHandler for ScreenMainEventHandler {
    fn on_click(&self, _point: TouchPoint) {
        crate::pinetimers::transition::spawn(Box::new(ScreenPoes::new())).unwrap();
    }
}

impl<COLOR : RgbColor + Send + Debug> Screen<COLOR> for ScreenMain<COLOR>
where
    Display<COLOR>: DisplaySupported<COLOR>,
{
    fn new() -> ScreenMain<COLOR> {
        ScreenMain {
            event_handler: Arc::new(ScreenMainEventHandler {}),
            _marker: PhantomData,
        }
    }

    fn get_event_handler(&self) -> Arc<dyn TouchPanelEventHandler> { 
        return self.event_handler.clone();
    }

    fn draw(&self, display: &mut Display<COLOR>, devicestate: &DeviceState) {
        display.clear(COLOR::WHITE).unwrap();
        let mut character_style = MonoTextStyle::new(&FONT_10X20, COLOR::BLACK);
        character_style.set_background_color(Some(COLOR::WHITE));
        Text::with_baseline("MAIN SCREEN", Point::new(0, 0), character_style, Baseline::Top)
            .draw(display)
            .unwrap();

        Text::with_baseline(&format!("{:?}", devicestate.battery), Point::new(0, 20), character_style, Baseline::Top)
            .draw(display)
            .unwrap();
    }
}
