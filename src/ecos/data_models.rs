// ecos/data_models.rs
#![allow(non_snake_case)]

use base64::engine::general_purpose;
use base64::Engine;
use rocket::serde::json::serde_json;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde", tag = "jwt")]
pub struct Claims {
    exp: u64,
}

impl Claims {
    #[cfg(not(test))]
    pub fn current_time() -> std::time::SystemTime {
        std::time::SystemTime::now()
    }

    #[cfg(test)]
    pub fn current_time() -> std::time::SystemTime {
        // NOTE: this will affect the tests in client.rs
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1735677779)
    }

    pub fn from_token(token: &str) -> Self {
        let parts = token.split('.').collect::<Vec<&str>>();
        if parts.len() != 3 {
            panic!("Invalid token format");
        }
        let decoded_bytes = general_purpose::STANDARD_NO_PAD
            .decode(parts[1])
            .expect("Invalid token");
        let decoded_str = String::from_utf8(decoded_bytes);
        serde_json::from_str(&decoded_str.unwrap()).expect("Invalid format for Claims")
    }
    pub fn is_expired(&self) -> bool {
        let now = Self::current_time();
        let now = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        self.exp < now
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde", tag = "ecos")]
pub struct LoginRequest {
    pub _t: u64,
    pub clientType: String,
    pub clientVersion: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde", tag = "ecos")]
pub struct LoginResponse {
    pub code: i32,
    pub message: String,
    pub success: bool,
    pub data: Option<LoginData>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde", tag = "ecos")]
pub struct LoginData {
    pub accessToken: String,
    pub refreshToken: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde", tag = "ecos")]
pub struct Device {
    pub deviceId: String,
    pub deviceAliasName: String,
    pub wifiSn: String,
    pub state: i32,
    pub weight: i32,
    pub temp: Option<i32>,
    pub icon: Option<String>,
    pub vpp: bool,
    pub master: i32,
    pub deviceSn: String,
    pub agentId: String,
    pub lon: f64,
    pub lat: f64,
    pub category: Option<String>,
    pub model: Option<String>,
    pub deviceType: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde", tag = "ecos")]
pub struct DevicesResponse {
    pub code: i32,
    pub message: String,
    pub success: bool,
    pub data: Vec<Device>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde", tag = "ecos")]
pub struct RunDataRequest {
    pub _t: u64,
    pub clientType: String,
    pub clientVersion: String,
    pub deviceId: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde", tag = "ecos")]
pub struct RunDataResponse {
    pub code: i32,
    pub message: String,
    pub success: bool,
    pub data: RunData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde", tag = "ecos")]
pub struct RunData {
    pub batterySoc: f32,
    pub batteryPower: f32,
    pub epsPower: f32,
    pub gridPower: f32,
    pub homePower: f32,
    pub meterPower: f32,
    pub solarPower: f32,
    pub sysRunMode: i32,
    pub isExistSolar: bool,
    pub sysPowerConfig: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde", tag = "ecos")]
pub struct ChargeModeSettingsResponse {
    pub code: i32,
    pub message: String,
    pub success: bool,
    pub data: ChargeModeSettings,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde", tag = "ecos")]
pub struct ChargeModeSettings {
    pub minCapacity: i32,
    pub chargeUseMode: i32,
    pub maxFeedIn: i32,
    pub epsBatteryMin: i32,
    pub dischargeToGridFlag: i32,
    pub selfSoc: i32,
    pub selfEpsBat: i32,
    pub selfFeedIn: i32,
    pub regularSoc: i32,
    pub regularEpsBat: i32,
    pub regularFeedIn: i32,
    pub backupSoc: i32,
    pub backupEpsBat: i32,
    pub backupFeedIn: i32,
    pub emsSoftwareVersion: String,
    pub dsp1SoftwareVersion: String,
    pub ratedPower: String,
    pub region: String,
    pub autoStrategy: i32,
    pub chargingList: Vec<ChargeSchedule>,
    pub dischargingList: Vec<ChargeSchedule>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde", tag = "ecos")]
pub struct ChargeSchedule {
    pub startHour: i32,
    pub startMinute: i32,
    pub endHour: i32,
    pub endMinute: i32,
    pub power: i32,
    pub abandonPv: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde", tag = "ecos")]
pub struct ChargeModeSettingsRequest {
    pub _t: u64,
    pub clientType: String,
    pub clientVersion: String,
    pub deviceId: String,
    pub chargeUseMode: i32,
    pub minCapacity: i32,
    pub maxFeedIn: i32,
    pub dischargeToGridFlag: i32,
    pub chargingList: Vec<ChargeSchedule>,
    pub dischargingList: Vec<ChargeSchedule>,
    pub epsBatteryMin: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_from_token() {
        // NOTE: this is a fake token
        let token = "eyJhbGciOiJIUzUxMiJ9.eyJqdGkiOiIxODYyMzg3NDMzNDQyOTA2MTEyIiwiYXV0aG9yaXRpZXMiOiJST0xFX1VTRVIiLCJzdWIiOiJ1c2VyQGV4YW1wbGUuY29tIiwiaWF0IjoxNzM1MTcyOTc5LCJleHAiOjE3MzU3Nzc3Nzl9.sdEfiGFJGomQbDexjWlkCiQjgrNpESjLXzi1DCGTTAhzL40SRbOU0lheouAUPjvQHb3dhhJXoeXPz4pgQQiOSA";
        let claims = Claims::from_token(token);
        assert_eq!(claims.exp, 1735777779);
        assert_eq!(claims.is_expired(), false);
    }
}
