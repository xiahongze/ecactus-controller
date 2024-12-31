use crate::config::AppConfig;
use crate::ecos::client::EcosClient;
use crate::ecos::data_models::{ChargeModeSettingsRequest, ChargeSchedule};
use crate::make_struct_with_time_device_info;
use rocket::log::private::{info, warn};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio;
use rocket::tokio::sync::Mutex;
use rocket::tokio::task::JoinHandle;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde", tag = "mode")]
pub enum ChargeMode {
    #[serde(rename = "conservative")]
    Conservative {
        battery_level: u8,
        duration: u64, // in minutes
    },
    #[serde(rename = "active")]
    Active {
        side_load: u32, // in watts
        duration: u64,  // in minutes
    },
    #[serde(rename = "self-sufficient")]
    SelfSufficient { battery_level: u8 },
}

pub struct AppState {
    pub current_mode: Mutex<ChargeMode>,
    pub expiration: Mutex<Option<Instant>>,
    pub background_task: Mutex<Option<JoinHandle<()>>>, // Track the active task
    pub app_config: AppConfig,
    pub ecos_client: Arc<EcosClient>,
}

impl AppState {
    /// update the current charge mode and expiration time
    pub async fn update_mode(&self, charge_mode: ChargeMode) {
        let mut current_mode = self.current_mode.lock().await;
        let mut expiration = self.expiration.lock().await;

        *current_mode = charge_mode;

        match *current_mode {
            ChargeMode::Conservative { duration, .. } => {
                *expiration = Some(Instant::now() + Duration::from_secs(duration * 60));
            }
            ChargeMode::Active { duration, .. } => {
                *expiration = Some(Instant::now() + Duration::from_secs(duration * 60));
            }
            _ => {
                *expiration = None;
            }
        }
    }

    /// Cancel the current background task if it exists
    pub async fn cancel_task(&self) {
        if let Some(task) = self.background_task.lock().await.take() {
            task.abort(); // Cancel the task
            info!(target: "app", "Task cancelled");
        }
    }

    /// Reset to default charge mode
    pub async fn reset_mode(&self) {
        info!(target: "app", "Resetting to default charge mode");

        self.update_mode(ChargeMode::SelfSufficient {
            battery_level: self.app_config.minCapacity as u8,
        })
        .await;

        self.update_charge_mode(0, Some(self.app_config.minCapacity), None)
            .await;
    }

    pub async fn update_charge_mode(
        &self,
        charge_use_mode: i32,
        battery_level: Option<i32>,
        side_load: Option<u32>,
    ) {
        let res = self
            .ecos_client
            .post_charge_mode_settings(make_struct_with_time_device_info!(
                ChargeModeSettingsRequest,
                deviceId: self.app_config.deviceId.clone(),
                chargeUseMode: charge_use_mode,
                minCapacity: battery_level.unwrap_or(self.app_config.minCapacity),
                maxFeedIn: self.app_config.maxFeedIn,
                dischargeToGridFlag: self.app_config.dischargeToGridFlag,
                chargingList: if charge_use_mode == 0 {self.app_config.chargingList.clone()} else {
                    let charge_power = self.compute_charge_power(side_load.unwrap_or(0)).await.unwrap_or(0.0);
                    info!(target: "app", "Charge power: {} W", charge_power);
                    vec![ChargeSchedule::from_now(
                        (self.app_config.checkInterval / 60) as i64,
                        charge_power as i32,
                    )]
                },
                dischargingList: self.app_config.dischargingList.clone(),
                epsBatteryMin: self.app_config.epsBatteryMin
            ))
            .await;
        if let Err(e) = res {
            warn!("Failed to update charge mode: {:?}", e);
            return;
        }
    }

    /// Start a background task to reset the charge mode
    pub async fn start_task(state: &Arc<AppState>) {
        state.cancel_task().await;

        let state_clone = state.clone();
        let task = tokio::spawn(async move {
            // release the lock immediately after cloning
            let current_mode = state_clone.current_mode.lock().await.clone();
            match current_mode {
                ChargeMode::SelfSufficient { battery_level } => {
                    info!(target: "app", "Self-sufficient mode: {}%", battery_level);
                    state_clone
                        .update_charge_mode(0, Some(battery_level as i32), None)
                        .await;
                }
                ChargeMode::Conservative {
                    duration,
                    battery_level,
                } => {
                    info!(target: "app", "Conservative mode: {}%, {} mins", battery_level, duration);
                    state_clone
                        .update_charge_mode(0, Some(battery_level as i32), None)
                        .await;
                    tokio::time::sleep(Duration::from_secs(duration * 60)).await;
                    info!(target: "app", "Conservative mode expired");
                    state_clone.reset_mode().await;
                }
                ChargeMode::Active {
                    duration,
                    side_load,
                } => {
                    info!(target: "app", "Active mode: side-load {} W, {} mins", side_load, duration);
                    let now = Instant::now();
                    let expiration = now + Duration::from_secs(duration * 60);
                    while Instant::now() < expiration {
                        state_clone
                            .update_charge_mode(1, None, Some(side_load))
                            .await;
                        tokio::time::sleep(Duration::from_secs(
                            state_clone.app_config.checkInterval,
                        ))
                        .await;
                        info!(target: "app", "Active mode: {} min left", expiration.duration_since(Instant::now()).as_secs() / 60);
                    }
                    state_clone.reset_mode().await;
                }
            }
        });

        *state.background_task.lock().await = Some(task);
    }

    /// Compute the charge power based on the current state
    /// This is a simplified implementation that only works for my home configuration
    /// where I have two identical PV inverters and one of them is connected to the battery.
    /// The side load is the power consumed by the house that is not monitored by the battery.
    /// Charge power is non-negative and capped at 5000W.
    pub async fn compute_charge_power(
        &self,
        side_load: u32,
    ) -> Result<f32, Box<dyn std::error::Error + Send + Sync>> {
        // the implementation only works for my home configuration where
        // I have two identical PV inverters and one of them is connected to the battery
        let run_data = self
            .ecos_client
            .get_run_data(self.app_config.deviceId.clone())
            .await?;
        let total_pv = run_data.data.solarPower * 2.0;
        let total_load = run_data.data.homePower + run_data.data.epsPower + side_load as f32;
        Ok(f32::max(f32::min(5000.0, total_pv - total_load), 0.0))
    }
}
