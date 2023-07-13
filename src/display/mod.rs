mod manager;

use embedded_graphics::{
    primitives::{Rectangle}
};
use esp_idf_hal::spi::{SpiDeviceDriver, SpiDriver};
use epd_waveshare::{
    epd7in5_v2::{Epd7in5},
    graphics::{VarDisplay},
    color::{Color}
};
use esp_idf_hal::{delay, gpio};
use esp_idf_hal::gpio::{Input, Output, PinDriver};

pub type Epd<'a> = Epd7in5<
    SpiDeviceDriver<'a,
        SpiDriver<'a>>,
    PinDriver<'a, gpio::AnyOutputPin, Output>,
    PinDriver<'a, gpio::AnyInputPin, Input>,
    PinDriver<'a, gpio::AnyOutputPin, Output>,
    PinDriver<'a, gpio::AnyOutputPin, Output>, delay::Ets
>;

pub struct DisplayManager<'a> {
    display: VarDisplay<'a, Color>,
    driver: SpiDeviceDriver<'a, SpiDriver<'a>>,
    epd: Epd<'a>,
    rect: DisplayRect,
}

pub struct DisplayRect {
    pub viewport: Rectangle,
    pub current_weather: Rectangle,
    pub weather_icon: Rectangle,
    pub current_temp: Rectangle,
    pub feels_like: Rectangle,
    pub current_temp_unit: Rectangle,
    pub date_location: Rectangle,
    pub forecast: Rectangle,
}

pub struct DisplayManagerPins {
    pub sclk: gpio::AnyOutputPin,
    pub sdo: gpio::AnyOutputPin,
    pub cs: gpio::AnyOutputPin,
    pub busy: gpio::AnyInputPin,
    pub dc: gpio::AnyOutputPin,
    pub rst: gpio::AnyOutputPin,
}


