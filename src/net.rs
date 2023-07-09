use anyhow::{Result};
use crate::config::CONFIG;
use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{EspWifi, BlockingWifi,};
use log::info;

pub fn connect_wifi() -> Result<Box<EspWifi<'static>>> {
    let app_config = CONFIG;
    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut esp_wifi = EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs))?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;


    wifi.set_configuration(&Configuration::Client(ClientConfiguration{
        ssid: app_config.wifi_ssid.into(),
        password: app_config.wifi_psk.into(),
        ..Default::default()
    }))?;

    info!("Starting wifi...");
    wifi.start()?;

    info!("Connecting to {:?}", app_config.wifi_ssid);
    wifi.connect()?;

    info!("Waiting for DHCP lease...");
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("Wifi DHCP info: {:?}", ip_info);


    Ok(Box::new(esp_wifi))
}