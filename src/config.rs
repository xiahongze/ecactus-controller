#![allow(non_snake_case)]
use crate::ecos::data_models::ChargeSchedule;
use rocket::serde::de::DeserializeOwned;
use rocket::serde::Deserialize;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    pub ecos: EcosConfig,
    pub app: AppConfig,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct EcosConfig {
    pub user: String,
    pub password: String,
    pub base_url: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", default)]
pub struct AppConfig {
    pub deviceId: String,
    pub checkInterval: u64, // in seconds
    pub chargeUseMode: i32,
    pub minCapacity: i32,
    pub maxFeedIn: i32,
    pub dischargeToGridFlag: i32,
    pub chargingList: Vec<ChargeSchedule>,
    pub dischargingList: Vec<ChargeSchedule>,
    pub epsBatteryMin: i32,
}

impl AppConfig {
    pub fn new() -> Self {
        AppConfig {
            deviceId: "123456".to_string(),
            checkInterval: 60 * 15, // 15 minutes
            chargeUseMode: 0,
            minCapacity: 10,
            maxFeedIn: 100,
            dischargeToGridFlag: 0,
            chargingList: vec![],
            dischargingList: vec![],
            epsBatteryMin: 10,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig::new()
    }
}

pub fn read_config<T: DeserializeOwned>(path: &str) -> T {
    let content = std::fs::read_to_string(path).expect("Failed to read config file");
    toml::from_str(&content).expect("Failed to parse config")
}
