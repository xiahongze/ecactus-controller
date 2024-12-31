// ecos/client.rs
use reqwest::{Client, Response, StatusCode};
use rocket::tokio::sync::Mutex;
use std::sync::Arc;

use crate::ecos::data_models::{
    ChargeModeSettingsRequest, ChargeModeSettingsResponse, Claims, DevicesResponse, LoginRequest,
    LoginResponse, RunDataRequest, RunDataResponse,
};

pub struct EcosClient {
    user: String,
    password: String,
    token: Arc<Mutex<Option<String>>>,
    base_url: String,
    client: Client,
    retries: u8,
}

#[macro_export]
macro_rules! make_struct_with_time_device_info {
    ($struct_name:ident, $($field_name:ident: $field_value:expr),*) => {
        $struct_name {
            _t: EcosClient::get_epoch_time(),
            clientType: "BROWSER".to_string(),
            clientVersion: "1.0".to_string(),
            $(
                $field_name: $field_value,
            )*
        }
    };
}

macro_rules! make_query_with_time_device_info {
    ($($field_name:literal: $field_value:expr),*) => {
        &[
            ("_t", Self::get_epoch_time().to_string()),
            ("clientType", "BROWSER".to_string()),
            ("clientVersion", "1.0".to_string()),
            $(
                ($field_name, $field_value),
            )*
        ]
    };
}

#[allow(dead_code)]
impl EcosClient {
    pub fn new(user: String, password: String, base_url: String) -> Self {
        EcosClient {
            user,
            password,
            token: Arc::new(Mutex::new(None)),
            base_url,
            client: Client::new(),
            retries: 3,
        }
    }

    pub fn get_epoch_time() -> u64 {
        let now = std::time::SystemTime::now();
        now.duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    pub async fn login(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let login_request = make_struct_with_time_device_info!(
            LoginRequest,
            email: self.user.clone(),
            password: self.password.clone()
        );

        let res = self
            .client
            .post(format!("{}/client/guide/login", self.base_url))
            .json(&login_request)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to login (status: {})", res.status()),
            )));
        }

        let login_response: LoginResponse = res.json().await?;

        if let Some(data) = login_response.data {
            let mut token = self.token.lock().await;
            *token = Some(data.accessToken);
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to login (success=false)",
            )))
        }
    }

    async fn retry_request<F>(
        &self,
        req_builder_func: F,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn() -> reqwest::RequestBuilder,
    {
        let mut retries = self.retries;

        let token_is_invalid = {
            let token = self.token.lock().await;
            token
                .clone()
                .map_or_else(|| true, |t| Claims::from_token(&t).is_expired())
        };

        if token_is_invalid {
            self.login().await?;
        }

        loop {
            let token = self.token.lock().await;
            let res = req_builder_func()
                .header("Authorization", token.as_ref().unwrap())
                .send()
                .await?;

            if res.status().is_success() {
                return Ok(res);
            }

            if res.status() == StatusCode::UNAUTHORIZED && retries > 0 {
                retries -= 1;
                self.login().await?;
            } else {
                return Ok(res);
            }
        }
    }

    pub async fn get_devices(
        &self,
    ) -> Result<DevicesResponse, Box<dyn std::error::Error + Send + Sync>> {
        let res = self
            .retry_request(|| {
                self.client
                    .get(format!("{}/client/home/device/list", self.base_url))
                    .query(make_query_with_time_device_info!())
            })
            .await?;

        let devices_response: DevicesResponse = res.json().await?;

        Ok(devices_response)
    }

    pub async fn get_run_data(
        &self,
        device_id: String,
    ) -> Result<RunDataResponse, Box<dyn std::error::Error + Send + Sync>> {
        let run_data_request = make_struct_with_time_device_info!(
            RunDataRequest,
            deviceId: device_id
        );

        let res = self
            .retry_request(|| {
                self.client
                    .post(format!("{}/client/home/now/device/runData", self.base_url))
                    .json(&run_data_request)
            })
            .await?;

        let run_data_response: RunDataResponse = res.json().await?;

        Ok(run_data_response)
    }

    pub async fn get_charge_mode_settings(
        &self,
        device_id: &str,
    ) -> Result<ChargeModeSettingsResponse, Box<dyn std::error::Error + Send + Sync>> {
        let res = self
            .retry_request(|| {
                self.client
                    .get(format!("{}/client/customize/info", self.base_url))
                    .query(make_query_with_time_device_info!("deviceId": device_id.to_string()))
            })
            .await?;

        let charge_mode_settings_response: ChargeModeSettingsResponse = res.json().await?;

        Ok(charge_mode_settings_response)
    }

    pub async fn post_charge_mode_settings(
        &self,
        charge_mode_settings_request: ChargeModeSettingsRequest,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let res = self
            .retry_request(|| {
                self.client
                    .post(format!("{}/client/customize/info", self.base_url))
                    .json(&charge_mode_settings_request)
            })
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to post charge mode settings",
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::tokio;

    async fn get_client() -> EcosClient {
        let client = EcosClient::new(
            std::env::var("ECOS_USER").unwrap_or_else(|_| "user".to_string()),
            std::env::var("ECOS_PASSWORD").unwrap_or_else(|_| "password".to_string()),
            "https://api-ecos-au.weiheng-tech.com/api".to_string(),
        );
        // Set the token to the value of the APP_TOKEN environment variable to test the APIs
        client
            .token
            .lock()
            .await
            .replace(std::env::var("ECOS_TOKEN").unwrap_or_else(|_| "token".to_string()));
        client
    }

    #[tokio::test]
    async fn test_login() {
        let client = get_client().await;
        let res = client.login().await;
        println!("{:?}", res);
    }

    #[tokio::test]
    async fn test_get_devices() {
        let client = get_client().await;
        let devices = client.get_devices().await.unwrap();
        println!("{:?}", devices);
    }

    #[tokio::test]
    async fn test_get_run_data() {
        let client = get_client().await;
        let devices = client.get_devices().await.unwrap();
        let device_id = devices.data[0].deviceId.clone();
        let run_data = client.get_run_data(device_id).await.unwrap();
        println!("{:?}", run_data);
    }

    #[tokio::test]
    async fn test_get_charge_mode_settings() {
        let client = get_client().await;
        let devices = client.get_devices().await.unwrap();
        let device_id = devices.data[0].deviceId.clone();
        let charge_mode_settings = client.get_charge_mode_settings(&device_id).await.unwrap();
        println!("{:?}", charge_mode_settings);
    }

    #[tokio::test]
    async fn test_post_charge_mode_settings() {
        let client = get_client().await;
        let devices = client.get_devices().await.unwrap();
        let device_id = devices.data[0].deviceId.clone();
        let charge_mode_settings = client.get_charge_mode_settings(&device_id).await.unwrap();
        let mut charge_mode_settings_request = ChargeModeSettingsRequest {
            _t: EcosClient::get_epoch_time(),
            clientType: "BROWSER".to_string(),
            clientVersion: "1.0".to_string(),
            deviceId: device_id,
            chargeUseMode: charge_mode_settings.data.chargeUseMode,
            minCapacity: charge_mode_settings.data.minCapacity,
            maxFeedIn: charge_mode_settings.data.maxFeedIn,
            dischargeToGridFlag: charge_mode_settings.data.dischargeToGridFlag,
            chargingList: charge_mode_settings.data.chargingList,
            dischargingList: charge_mode_settings.data.dischargingList,
            epsBatteryMin: charge_mode_settings.data.epsBatteryMin,
        };
        charge_mode_settings_request.minCapacity = 10;
        let res = client
            .post_charge_mode_settings(charge_mode_settings_request)
            .await;
        println!("{:?}", res);
    }
}
