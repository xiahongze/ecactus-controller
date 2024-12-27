use ecactus_controller::config::AppConfig;
use ecactus_controller::state::AppState;
use ecactus_controller::{models, routes};
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::Client;
use rocket::serde::json::{json, serde_json};
use rocket::{routes, tokio};
use std::sync::Arc;
fn get_app_config() -> AppConfig {
    let config = AppConfig {
        default_mode: "self-sufficient".to_string(),
        default_battery_level: 10,
    };
    config
}

fn get_self_sufficient_app_state(config: &AppConfig) -> Arc<AppState> {
    let app_state = Arc::new(AppState {
        current_mode: tokio::sync::Mutex::new(models::ChargeMode::SelfSufficient {
            battery_level: config.default_battery_level,
        }),
        expiration: tokio::sync::Mutex::new(None),
        background_task: tokio::sync::Mutex::new(None),
    });
    app_state
}
async fn create_client(
    app_state: Arc<AppState>,
    config: AppConfig,
    routes: Vec<rocket::Route>,
) -> Client {
    let rocket = rocket::build()
        .manage(app_state)
        .manage(config)
        .mount("/", routes);

    Client::tracked(rocket)
        .await
        .expect("valid rocket instance")
}

#[rocket::async_test]
async fn test_get_charge_mode() {
    let config = get_app_config();

    let app_state = get_self_sufficient_app_state(&config);

    let client = create_client(app_state, config, routes![routes::charge_mode::get_mode]).await;

    let response = client.get("/charge-mode").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.expect("response into string");
    let charge_mode: models::ChargeMode = serde_json::from_str(&body).expect("parse charge mode");

    if let models::ChargeMode::SelfSufficient { battery_level } = charge_mode {
        assert_eq!(battery_level, 10);
    } else {
        panic!("Expected SelfSufficient mode");
    }
}

#[rocket::async_test]
async fn test_post_charge_mode_conservative() {
    let config = get_app_config();

    let app_state = get_self_sufficient_app_state(&config);

    let client = create_client(app_state, config, routes![routes::charge_mode::set_mode]).await;

    let payload = json!({
        "mode": "conservative",
        "battery_level": 80,
        "duration": 60
    });

    let response = client
        .post("/charge-mode")
        .header(ContentType::JSON)
        .body(payload.to_string())
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let state = client.rocket().state::<Arc<AppState>>().unwrap();
    let current_mode = state.current_mode.lock().await.clone();

    if let models::ChargeMode::Conservative {
        battery_level,
        duration,
        ..
    } = current_mode
    {
        assert_eq!(battery_level, 80);
        assert_eq!(duration, 60);
    } else {
        panic!("Expected Conservative mode");
    }
}

#[rocket::async_test]
async fn test_put_charge_mode_reset() {
    let config = get_app_config();

    let app_state = Arc::new(AppState {
        current_mode: tokio::sync::Mutex::new(models::ChargeMode::Conservative {
            battery_level: 80,
            duration: 60,
        }),
        expiration: tokio::sync::Mutex::new(Some(std::time::Instant::now())),
        background_task: tokio::sync::Mutex::new(None),
    });

    let client = create_client(app_state, config, routes![routes::charge_mode::reset_mode]).await;

    let response = client.put("/charge-mode/reset").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let state = client.rocket().state::<Arc<AppState>>().unwrap();
    let current_mode = state.current_mode.lock().await.clone();

    if let models::ChargeMode::SelfSufficient { battery_level } = current_mode {
        assert_eq!(battery_level, 10);
    } else {
        panic!("Expected SelfSufficient mode");
    }
}
