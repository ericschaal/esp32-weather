mod config;
mod wifi;
mod owm;
mod display;
mod http_client;
mod icons;
mod chart;

use std::thread;
use std::time::Duration;
use anyhow::{Result};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;

use crate::display::{DisplayManager, DisplayManagerPins};
use crate::owm::api::fetch_owm_report;
use crate::wifi::WifiManager;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    let pins = peripherals.pins;

    let mut wifi = WifiManager::new(peripherals.modem, sys_loop, nvs)?;
    wifi.connect()?;

    let weather = fetch_owm_report()?;

    let spi = peripherals.spi2;
    let sclk = pins.gpio19;
    let sdo = pins.gpio23;
    let cs = pins.gpio17;
    let busy = pins.gpio18;
    let dc = pins.gpio22;
    let rst = pins.gpio21;

    let mut buffer = DisplayManager::new_buffer();
    let mut display = DisplayManager::new(
        spi,
        DisplayManagerPins {
            sclk: sclk.into(),
            sdo: sdo.into(),
            cs: cs.into(),
            busy: busy.into(),
            dc: dc.into(),
            rst: rst.into()
        },
        &mut buffer
    )?;

    display.draw_weather_report(weather)?;


    loop {
        thread::sleep(Duration::from_secs(1));
    }

}


