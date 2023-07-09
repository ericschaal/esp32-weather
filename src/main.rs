mod config;
mod http;
mod net;
mod owm_api;

use crate::net::connect_wifi;

use anyhow::{Result};
use esp_idf_sys as _;
use log::*;
use crate::owm_api::fetch_owm_report; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported


fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let _wifi = connect_wifi()?;
    let weather = fetch_owm_report(45.5019, 73.5674)?;

    info!("Weather report: {:?}", weather);

    Ok(())
}

