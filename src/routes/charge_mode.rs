use crate::state::AppState;
use crate::state::ChargeMode;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::{get, post, put, routes, State};
use std::sync::Arc;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde", tag = "mode")]
pub struct Message {
    message: String,
}

#[post("/charge-mode", data = "<charge_mode>")]
pub async fn set_mode(
    charge_mode: Json<ChargeMode>,
    state: &State<Arc<AppState>>,
) -> Json<Message> {
    state.update_mode(charge_mode.into_inner()).await;
    AppState::start_task(&**state).await;

    Json(Message {
        message: "Charge mode update request sent".to_string(),
    })
}

#[put("/charge-mode/reset")]
pub async fn reset_mode(state: &State<Arc<AppState>>) -> Json<Message> {
    state.cancel_task().await;
    state.reset_mode().await;

    Json(Message {
        message: "Charge mode reset".to_string(),
    })
}

#[get("/charge-mode")]
pub async fn get_mode(state: &State<Arc<AppState>>) -> Json<ChargeMode> {
    let current_mode = state.current_mode.lock().await.clone();
    Json(current_mode)
}

pub fn routes() -> Vec<rocket::Route> {
    routes![set_mode, reset_mode, get_mode]
}
