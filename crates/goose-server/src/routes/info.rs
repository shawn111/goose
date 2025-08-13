use axum::{Router, routing::get, Json};
use serde::{Serialize, Deserialize};
use etcetera::{choose_app_strategy, AppStrategy};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// Define the struct to match the JSON response for /info
#[derive(Serialize, Deserialize)]
pub struct InfoResponse {
    version: String,
    config_file: String,
    sessions_dir: String,
    logs_dir: String,
    config_values: Option<std::collections::BTreeMap<String, String>>,
}

pub async fn get_info_handler() -> Json<InfoResponse> {
    let app_strategy = etcetera::choose_app_strategy(etcetera::AppStrategyArgs::default()).expect("Failed to choose app strategy");
    let config_dir = app_strategy.config_dir();
    let data_dir = app_strategy.data_dir();

    let config_file = config_dir.join("config.yaml");
    let sessions_dir = data_dir.join("sessions");
    let logs_dir = data_dir.join("logs");

    Json(InfoResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        config_file: config_file.to_string_lossy().into_owned(),
        sessions_dir: sessions_dir.to_string_lossy().into_owned(),
        logs_dir: logs_dir.to_string_lossy().into_owned(),
        config_values: None, // For now, we'll leave this as None
    })
}

pub fn routes() -> Router {
    Router::new().route("/info", get(get_info_handler))
}
