use crate::ecos::data_models::{ChargeModeSettingsResponse, DevicesResponse, RunDataResponse};
use crate::state::AppState;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::{get, routes, State};
use std::sync::Arc;

#[get("/devices")]
async fn get_devices(
    state: &State<Arc<AppState>>,
) -> Result<Json<DevicesResponse>, Custom<String>> {
    state
        .ecos_client
        .get_devices()
        .await
        .map(Json)
        .map_err(|e| Custom(rocket::http::Status::InternalServerError, e.to_string()))
}

#[get("/run-data?<device_id>")]
async fn get_run_data(
    state: &State<Arc<AppState>>,
    device_id: Option<String>,
) -> Result<Json<RunDataResponse>, Custom<String>> {
    let device_id = device_id.unwrap_or_else(|| state.app_config.deviceId.clone());
    state
        .ecos_client
        .get_run_data(device_id)
        .await
        .map(Json)
        .map_err(|e| Custom(rocket::http::Status::InternalServerError, e.to_string()))
}

#[get("/charge-mode-settings?<device_id>")]
async fn get_charge_mode_settings(
    state: &State<Arc<AppState>>,
    device_id: Option<String>,
) -> Result<Json<ChargeModeSettingsResponse>, Custom<String>> {
    let device_id = device_id.unwrap_or_else(|| state.app_config.deviceId.clone());
    state
        .ecos_client
        .get_charge_mode_settings(&device_id)
        .await
        .map(Json)
        .map_err(|e| Custom(rocket::http::Status::InternalServerError, e.to_string()))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![get_devices, get_run_data, get_charge_mode_settings]
}
