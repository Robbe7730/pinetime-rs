use nrf52832_hal::twim::Twim;
use nrf52832_hal::pac::TWIM1;

pub struct TouchPanel<HANDLER>
where
    HANDLER: TouchPanelEventHandler,
{
    twim: Twim<TWIM1>,
    event_handler: Option<HANDLER>,
}

#[derive(Debug)]
pub enum GestureType {
    NoGesture,
    SlideDown,
    SlideUp,
    SlideLeft,
    SlideRight,
    ClickSingle,
    ClickDouble,
    ClickLong,
}

#[derive(Debug)]
pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
}

impl From<u8> for GestureType {
    fn from(value: u8) -> GestureType {
        match value {
            0x00 => GestureType::NoGesture,
            0x01 => GestureType::SlideDown,
            0x02 => GestureType::SlideUp,
            0x03 => GestureType::SlideLeft,
            0x04 => GestureType::SlideRight,
            0x05 => GestureType::ClickSingle,
            0x0b => GestureType::ClickDouble, // Could not get this to activate...
            0x0c => GestureType::ClickLong,
            x => panic!("Unknown GestureType {}", x),
        }
    }
}

// Most precise function gets called
pub trait TouchPanelEventHandler {
    fn on_slide_down(&self, point: TouchPoint) {
        self.on_slide(point);
    }

    fn on_slide_up(&self, point: TouchPoint) {
        self.on_slide(point);
    }

    fn on_slide_left(&self, point: TouchPoint) {
        self.on_slide(point);
    }

    fn on_slide_right(&self, point: TouchPoint) {
        self.on_slide(point);
    }

    fn on_click_single(&self, point: TouchPoint) {
        self.on_click(point);
    }

    fn on_click_double(&self, point: TouchPoint) {
        self.on_slide(point);
    }

    fn on_click_long(&self, point: TouchPoint) {
        self.on_slide(point);
    }

    fn on_slide(&self, point: TouchPoint) {
        self.on_event(point);
    }

    fn on_click(&self, point: TouchPoint) {
        self.on_event(point)
    }

    fn on_event(&self, _point: TouchPoint) {}
}

impl<HANDLER : TouchPanelEventHandler> TouchPanel<HANDLER> {
    pub fn new(twim: Twim<TWIM1>, event_handler: Option<HANDLER>) -> Self {
        TouchPanel {
            twim,
            event_handler
        }
    }

    pub fn handle_interrupt(&mut self) {
        let mut buffer = [0; 63];
        self.twim.read(0x15, &mut buffer).unwrap();

        // Reading the touch points does not seem correct, there appears to
        // be only ever one touch point
        let gesture: GestureType = buffer[1].into();

        let touchpoint = TouchPoint {
            x: (((buffer[3] & 0xf) as u16) << 8) | buffer[4] as u16,
            y: (((buffer[5] & 0xf) as u16) << 8) | buffer[6] as u16,
        };

        if let Some(handler) = &self.event_handler {
            match gesture {
                GestureType::SlideDown => handler.on_slide_down(touchpoint),
                GestureType::SlideUp => handler.on_slide_up(touchpoint),
                GestureType::SlideLeft => handler.on_slide_left(touchpoint),
                GestureType::SlideRight => handler.on_slide_right(touchpoint),
                GestureType::ClickSingle => handler.on_click_single(touchpoint),
                GestureType::ClickDouble => handler.on_click_double(touchpoint),
                GestureType::ClickLong => handler.on_click_long(touchpoint),
                _ => handler.on_event(touchpoint),
            }
        }
    }
}
