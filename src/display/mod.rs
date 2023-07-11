mod manager;

use esp_idf_hal::spi::{SpiDeviceDriver, SpiDriver};
use epd_waveshare::{
    epd7in5_v2::{Epd7in5},
    graphics::{VarDisplay},
    prelude::*
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
    epd: Epd<'a>
}

pub struct DisplayManagerPins {
    pub sclk: gpio::AnyOutputPin,
    pub sdo: gpio::AnyOutputPin,
    pub cs: gpio::AnyOutputPin,
    pub busy: gpio::AnyInputPin,
    pub dc: gpio::AnyOutputPin,
    pub rst: gpio::AnyOutputPin,
}


