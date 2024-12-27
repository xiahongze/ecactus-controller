use crate::models::ChargeMode;
use rocket::tokio;
use rocket::tokio::sync::Mutex;
use rocket::tokio::task::JoinHandle;
use std::sync::Arc;
use std::time::{Duration, Instant};
pub struct AppState {
    pub current_mode: Mutex<ChargeMode>,
    pub expiration: Mutex<Option<Instant>>,
    pub background_task: Mutex<Option<JoinHandle<()>>>, // Track the active task
}

impl AppState {
    /// update the current charge mode and expiration time
    pub async fn update_mode(&self, charge_mode: ChargeMode) {
        let mut current_mode = self.current_mode.lock().await;
        let mut expiration = self.expiration.lock().await;

        *current_mode = charge_mode;

        match *current_mode {
            ChargeMode::Conservative { duration, .. } => {
                *expiration = Some(Instant::now() + Duration::from_secs(duration as u64 * 60));
            }
            ChargeMode::Active { duration, .. } => {
                *expiration = Some(Instant::now() + Duration::from_secs(duration as u64 * 60));
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
        }
    }

    /// Reset to default charge mode
    pub async fn reset_mode(&self, default_battery_level: u8) {
        self.cancel_task().await;

        let mut current_mode = self.current_mode.lock().await;
        let mut expiration = self.expiration.lock().await;

        *current_mode = ChargeMode::SelfSufficient {
            battery_level: default_battery_level,
        };
        *expiration = None;
    }

    /// Start a background task to reset the charge mode
    pub async fn start_task(state: &Arc<AppState>, battery_level: u8) {
        state.cancel_task().await;

        let state_clone = state.clone();
        let task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(60)).await;
            let mut current_mode = state_clone.current_mode.lock().await;
            let mut expiration = state_clone.expiration.lock().await;

            *current_mode = ChargeMode::SelfSufficient { battery_level };
            *expiration = None;
        });

        *state.background_task.lock().await = Some(task);
    }
}
