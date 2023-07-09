use anyhow::{Result};
use crate::config::CONFIG;
use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{EspWifi, BlockingWifi,};
use log::info;


pub struct WifiManager {
    wifi: BlockingWifi<EspWifi<'static>>,
}

impl WifiManager {
    pub fn new() -> Result<Self> {
        let peripherals = Peripherals::take().unwrap();
        let sysloop = EspSystemEventLoop::take().unwrap();
        let nvs = EspDefaultNvsPartition::take().unwrap();

        let esp_wifi = EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs))?;
        let wifi = BlockingWifi::wrap(esp_wifi, sysloop)?;

        Ok(WifiManager {
            wifi
        })
    }

    pub fn connect(&mut self) -> Result<()> {
        let app_config = CONFIG;
        self.wifi.set_configuration(&Configuration::Client(ClientConfiguration{
            ssid: app_config.wifi_ssid.into(),
            password: app_config.wifi_psk.into(),
            ..Default::default()
        }))?;

        info!("Starting wifi...");
        self.wifi.start()?;

        info!("Connecting to {:?}", app_config.wifi_ssid);
        self.wifi.connect()?;

        info!("Waiting for DHCP lease...");
        self.wifi.wait_netif_up()?;

        let ip_info = self.wifi.wifi().sta_netif().get_ip_info()?;
        info!("Wifi DHCP info: {:?}", ip_info);

        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<()> {
        self.wifi.disconnect()?;
        self.wifi.stop()?;
        Ok(())
    }
}
