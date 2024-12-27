use rocket::serde::json::Json;
use rocket::State;
use rocket::{get, post, put};
use std::sync::Arc;

use crate::{config::AppConfig, models::ChargeMode, state::AppState};

#[post("/charge-mode", data = "<charge_mode>")]
pub async fn set_mode(
    charge_mode: Json<ChargeMode>,
    state: &State<Arc<AppState>>,
    config: &State<AppConfig>,
) {
    // Update the charge mode
    state.update_mode(charge_mode.into_inner()).await;

    // Start background task for reset
    AppState::start_task(&**state, config.default_battery_level).await;
}

#[put("/charge-mode/reset")]
pub async fn reset_mode(state: &State<Arc<AppState>>, config: &State<AppConfig>) {
    // Reset to default charge mode
    state.reset_mode(config.default_battery_level).await;
}

#[get("/charge-mode")]
pub async fn get_mode(state: &State<Arc<AppState>>) -> Json<ChargeMode> {
    let current_mode = state.current_mode.lock().await;
    Json(current_mode.clone())
}
