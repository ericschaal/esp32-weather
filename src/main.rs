mod config;
mod http;
mod wifi;
mod owm;
mod task;

use std::thread::sleep;
use std::time::Duration;

use anyhow::{Result};
use esp_idf_sys as _;
use log::*;
use crate::wifi::WifiManager;
use crate::owm::api::fetch_owm_report;

fn fetch_report_task() -> Result<()> {
    let mut wifi = WifiManager::new()?;
    wifi.connect()?;

    let weather = fetch_owm_report(45.5019, 73.5674).unwrap();
    info!("Weather report: {:?}", weather);

    wifi.disconnect()?;

    Ok(())
}

fn go_to_deep_sleep() {
    info!("Going to deep sleep");
    unsafe {
        esp_idf_sys::esp_deep_sleep(Duration::from_secs(10).as_micros() as u64)
    }
}

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    loop {
        fetch_report_task()?;
        go_to_deep_sleep();
        sleep(Duration::from_secs(1));
    }

}

