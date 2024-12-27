use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde", tag = "mode")]
pub enum ChargeMode {
    #[serde(rename = "conservative")]
    Conservative {
        battery_level: u8,
        duration: u32, // in minutes
    },
    #[serde(rename = "active")]
    Active {
        side_load: u32, // in watts
        duration: u32,  // in minutes
    },
    #[serde(rename = "self-sufficient")]
    SelfSufficient { battery_level: u8 },
}
