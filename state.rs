use crate::models::ChargeMode;
use std::sync::Mutex;
use std::time::Instant;

pub struct AppState {
    pub current_mode: Mutex<ChargeMode>,
    pub expiration: Mutex<Option<Instant>>,
}
