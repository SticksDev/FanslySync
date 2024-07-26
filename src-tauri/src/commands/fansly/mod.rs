use crate::{
    handlers::fansly::Fansly,
    structs::{FanslyAccountResponse, FanslyBaseResponse, SyncDataResponse},
};
use lazy_static::lazy_static;
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
pub async fn fansly_sync() -> Result<SyncDataResponse, String> {
    let fansly = FANSLY.lock().await;
    let response = fansly.sync().await;

    match response {
        Ok(response) => Ok(response),
        Err(e) => Err(e.to_string()),
    }
}
