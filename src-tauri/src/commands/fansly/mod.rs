use crate::{
    handlers::fansly::Fansly,
    structs::{FanslyAccountResponse, FanslyBaseResponse, SyncDataResponse},
};
use lazy_static::lazy_static;
use serde_json::Value;
use tokio::sync::Mutex;

lazy_static! {
    static ref FANSLY: Mutex<Fansly> = Mutex::new(Fansly::new(None));
}

#[tauri::command]
pub async fn fansly_set_token(token: Option<String>) {
    FANSLY.lock().await.set_token(token);
}

#[tauri::command]
pub async fn fansly_get_me() -> Result<FanslyBaseResponse<FanslyAccountResponse>, String> {
    let fansly = FANSLY.lock().await;
    let response = fansly.get_profile().await;

    match response {
        Ok(response) => Ok(response),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn fansly_sync(auto: bool) -> Result<SyncDataResponse, String> {
    let fansly = FANSLY.lock().await;
    let response = fansly.sync(auto).await;

    match response {
        Ok(response) => Ok(response),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn fansly_upload_auto_sync_data(
    data: SyncDataResponse,
    token: String,
) -> Result<(), String> {
    let fansly: tokio::sync::MutexGuard<Fansly> = FANSLY.lock().await;
    let response = fansly.upload_auto_sync_data(data, token).await;

    match response {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn fansly_check_sync_token(token: String) -> Result<Value, String> {
    let fansly: tokio::sync::MutexGuard<Fansly> = FANSLY.lock().await;
    let response = fansly.check_sync_token(token).await;

    match response {
        Ok(response) => Ok(response),
        Err(e) => Err(e.to_string()),
    }
}
