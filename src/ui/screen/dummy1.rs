use crate::ui::screen::{Screen, ScreenDummy2};
use crate::drivers::touchpanel::{TouchPanelEventHandler, TouchPoint};
use crate::drivers::display::{Display, DisplaySupported};

use embedded_graphics::prelude::{Point, Drawable};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::RgbColor;
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::text::{Text, Baseline};

use core::marker::PhantomData;
use core::fmt::Debug;

use alloc::sync::Arc;
use alloc::boxed::Box;

#[derive(Debug)]
pub struct ScreenDummy1<COLOR> {
    event_handler: Arc<ScreenDummy1EventHandler>,
    _marker: PhantomData<COLOR>
}

#[derive(Debug)]
pub struct ScreenDummy1EventHandler {}

impl TouchPanelEventHandler for ScreenDummy1EventHandler {
    fn on_slide_right(&self, _point: TouchPoint) {
        crate::pinetimers::transition::spawn(Box::new(ScreenDummy2::new())).unwrap();
    }
}

impl<COLOR : RgbColor + Send + Debug> Screen<COLOR> for ScreenDummy1<COLOR>
where
    Display<COLOR>: DisplaySupported<COLOR>,
{
    fn new() -> ScreenDummy1<COLOR> {
        ScreenDummy1 {
            event_handler: Arc::new(ScreenDummy1EventHandler {}),
            _marker: PhantomData,
        }
    }

    fn get_event_handler(&self) -> Arc<dyn TouchPanelEventHandler> { 
        return self.event_handler.clone();
    }

    fn draw(&self, display: &mut Display<COLOR>) {
        let mut character_style = MonoTextStyle::new(&FONT_10X20, COLOR::BLACK);
        character_style.set_background_color(Some(COLOR::WHITE));
        Text::with_baseline("Screen 1", Point::new(0, 0), character_style, Baseline::Top)
            .draw(display)
            .unwrap();
    }
}
