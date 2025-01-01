extern crate rocket;

mod config;
mod ecos;
mod routes;
mod state;

use crate::config::{read_config, Config};
use crate::state::{AppState, ChargeMode};
use rocket::tokio::sync::Mutex;
use std::sync::Arc;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let config_path = std::env::var("APP_CONFIG").unwrap_or_else(|_| "config.toml".to_string());
    let config: Config = read_config(&config_path);

    let ecos_client = Arc::new(ecos::client::EcosClient::new(
        config.ecos.user,
        config.ecos.password,
        config.ecos.base_url,
    ));
    let app_state = AppState {
        current_mode: Mutex::new(ChargeMode::SelfSufficient {
            battery_level: config.app.minCapacity as u8,
        }),
        expiration: Mutex::new(None),
        background_task: Mutex::new(None),
        app_config: config.app,
        ecos_client,
    };

    rocket::build()
        .mount("/", routes::charge_mode::routes())
        .mount("/ecos", routes::ecos::routes())
        .manage(Arc::new(app_state))
        .launch()
        .await?;

    Ok(())
}
