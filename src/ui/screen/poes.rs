use crate::ui::screen::{Screen, ScreenMain};
use crate::drivers::touchpanel::{TouchPanelEventHandler, TouchPoint};
use crate::drivers::display::{Display, DisplaySupported};
use crate::devicestate::DeviceState;

use embedded_graphics::pixelcolor::{RgbColor, PixelColor};
use embedded_graphics::prelude::{Drawable, Point};
use embedded_graphics::image::{Image, ImageRawLE, ImageDrawable};

use core::marker::PhantomData;
use core::fmt::Debug;

use alloc::sync::Arc;
use alloc::boxed::Box;

use tinybmp::Bmp;

#[derive(Debug)]
pub struct ScreenPoes<COLOR> {
    event_handler: Arc<ScreenPoesEventHandler>,
    _marker: PhantomData<COLOR>
}

#[derive(Debug)]
pub struct ScreenPoesEventHandler {}

impl TouchPanelEventHandler for ScreenPoesEventHandler {
    fn on_click(&self, _point: TouchPoint) {
        crate::pinetimers::transition::spawn(Box::new(ScreenMain::new())).unwrap();
    }
}

impl<'a, COLOR : RgbColor + Send + Debug> Screen<COLOR> for ScreenPoes<COLOR>
where
    Display<COLOR>: DisplaySupported<COLOR>,
    COLOR: From<<COLOR as PixelColor>::Raw>,
    ImageRawLE<'a, COLOR>: ImageDrawable<Color = COLOR>,
{
    fn new() -> ScreenPoes<COLOR> {
        ScreenPoes {
            event_handler: Arc::new(ScreenPoesEventHandler {}),
            _marker: PhantomData,
        }
    }

    fn get_event_handler(&self) -> Arc<dyn TouchPanelEventHandler> { 
        return self.event_handler.clone();
    }

    fn draw(&self, display: &mut Display<COLOR>, _devicestate: &DeviceState) {
        let bmp_data = include_bytes!("../../../poes565.bmp");
        let image = Bmp::<COLOR>::from_slice(bmp_data).unwrap();
        Image::new(&image, Point::new(0,0))
            .draw(display)
            .unwrap();
    }
}
