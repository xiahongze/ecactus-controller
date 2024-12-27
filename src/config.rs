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
#[serde(crate = "rocket::serde")]
pub struct AppConfig {
    pub default_mode: String,
    pub default_battery_level: u8,
}

pub fn read_config<T: DeserializeOwned>(path: &str) -> T {
    let content = std::fs::read_to_string(path).expect("Failed to read config file");
    toml::from_str(&content).expect("Failed to parse config")
}
