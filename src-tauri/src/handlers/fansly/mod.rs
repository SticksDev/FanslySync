// Create a simple module for handling the Fansly API, using reqwest to make requests to the API.
// This module will contain a struct Fansly, which will have a method to get the user's profile information.
use crate::structs::{
    FanslyAccountResponse, FanslyBaseResponse, FanslyBaseResponseList, FanslyFollowersResponse,
    FanslySubscriptionsResponse, Subscription, SyncDataResponse,
};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::Value;

pub struct Fansly {
    client: reqwest::Client,
    token: Option<String>,
}

impl Fansly {
    pub fn new(token: Option<String>) -> Self {
        let mut headers = HeaderMap::new();

        // Set the user agent to the FanslySync/0.1.0 tanner@fanslycreatorbot.com
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("FanslySync/0.1.0 tanner@fanslycreatorbot.com"),
        );

        // If we have a token, add it to the headers\
        if let Some(token) = &token {
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("{}", token)).unwrap(),
            );
        }

        // Set our default base url to https://apiv3.fansly.com/api/v1/
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        Self { client, token }
    }

    // Helper function to set our token on the fly
    pub fn set_token(&mut self, token: Option<String>) {
        self.token = token;

        // Re-create the client with the new token (if it exists)
        let mut headers = HeaderMap::new();

        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("FanslySync/0.1.0 tanner@fanslycreatorbot.com"),
        );

        // If we have a token, add it to the headers
        if let Some(token) = &self.token {
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("{}", token)).unwrap(),
            );
        }

        self.client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
    }

    pub async fn get_profile(
        &self,
    ) -> Result<FanslyBaseResponse<FanslyAccountResponse>, reqwest::Error> {
        let response = self
            .client
            .get("https://apiv3.fansly.com/api/v1/account/me")
            .send()
            .await?;

        if !response.status().is_success() {
            eprintln!("[sync::process::get_profile] No successful response from API. Setting error state.");
            return Err(response.error_for_status().unwrap_err());
        } else {
            println!("[sync::process::get_profile] Got successful response from API.");
        }

        let profile = response
            .json::<FanslyBaseResponse<FanslyAccountResponse>>()
            .await?;
        Ok(profile)
    }

    async fn fetch_followers(
        &self,
        account_id: &str,
        auth_token: &str,
        offset: u32,
    ) -> Result<FanslyBaseResponseList<FanslyFollowersResponse>, reqwest::Error> {
        let url = format!("https://apiv3.fansly.com/api/v1/account/{}/followers?ngsw-bypass=true&limit=100&offset={}", account_id, offset);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("{}", auth_token).parse().unwrap(),
        );
        headers.insert(
            reqwest::header::USER_AGENT,
            "FanslySync/1.0.0 (tanner@fanslycreatorbot.com)"
                .parse()
                .unwrap(),
        );

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        let response = self.client.get(url).headers(headers).send().await?;

        if !response.status().is_success() {
            eprintln!("[sync::process::fetch_followers] No successful response from API. Setting error state.");
            return Err(response.error_for_status().unwrap_err());
        }

        let followers: FanslyBaseResponseList<FanslyFollowersResponse> = response.json().await?;
        println!(
            "[sync::process::fetch_followers] Got {} followers from API.",
            followers.response.len()
        );

        Ok(followers)
    }

    async fn fetch_subscribers(
        &self,
        auth_token: &str,
        offset: u32,
    ) -> Result<Vec<Subscription>, reqwest::Error> {
        let url = format!("https://apiv3.fansly.com/api/v1/subscribers?status=3,4&limit=100&offset={}&ngsw-bypass=true", offset);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("{}", auth_token).parse().unwrap(),
        );
        headers.insert(
            reqwest::header::USER_AGENT,
            "FanslySync/1.0.0 (tanner@fanslycreatorbot.com)"
                .parse()
                .unwrap(),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        let response = self.client.get(url).headers(headers).send().await?;

        if !response.status().is_success() {
            log::error!("[sync::process::fetch_subscribers] No successful response from API. Setting error state.");
            let error = response.error_for_status().unwrap_err();
            return Err(error);
        }

        let subscriptions: FanslyBaseResponse<FanslySubscriptionsResponse> =
            response.json().await?;

        log::info!(
            "[sync::process::fetch_subscribers] Got {} subscribers from API.",
            subscriptions.response.subscriptions.len()
        );

        Ok(subscriptions.response.subscriptions)
    }

    async fn upload_sync_data(&self, data: SyncDataResponse) -> Result<String, reqwest::Error> {
        let url = "https://paste.fanslycreatorbot.com";

        // Convert passed data to bytes
        let json_string = serde_json::to_string(&data).unwrap();
        let data_as_bytes = json_string.as_bytes();

        let form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(data_as_bytes.to_vec())
                    .file_name("sync_data.json")
                    .mime_str("application/json")?,
            );

        // Create a new client and POST
        let response = self
            .client
            .post(url)
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            log::error!("Failed to upload sync data...");
            log::info!("Response: {:?}", response);
            return Err(response.error_for_status().unwrap_err());
        }

        let reply = response.text().await?;
        log::info!("Uploaded sync data successfully. Response: {}", reply);
        Ok(reply)
    }

    pub async fn upload_auto_sync_data(
        &self,
        data: SyncDataResponse,
        token: String,
    ) -> Result<(), reqwest::Error> {
        let url = "https://botapi.fanslycreatorbot.com/sync";

        // Set our content type to application/json
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        // Add our auth token to the headers
        headers.insert("Authorization", format!("{}", token).parse().unwrap());

        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&data)
            .send()
            .await?;

        if !response.status().is_success() {
            log::error!("Failed to upload sync data...");
            log::info!("Response: {:?}", response);
            return Err(response.error_for_status().unwrap_err());
        }

        log::info!("Uploaded sync data successfully.");
        Ok(())
    }

    pub async fn check_sync_token(&self, token: String) -> Result<Value, reqwest::Error> {
        // Check if the token is valid (GET /checkSyncToken with Authorization header)
        // If it is, return the data back from the API
        // If it isn't, return an error
        let url = "https://botapi.fanslycreatorbot.com/checkSyncToken";

        // Set our content type to application/json
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        // Add our auth token to the headers
        headers.insert("Authorization", format!("{}", token).parse().unwrap());

        let response = self.client.get(url).headers(headers).send().await;

        // If successful, return the data, otherwise return an error
        match response {
            Ok(response) => {
                if !response.status().is_success() {
                    log::error!("Failed to check sync token...");
                    log::info!("Response: {:?}", response);
                    return Err(response.error_for_status().unwrap_err());
                }

                let json: serde_json::Value = response.json().await?;
                Ok(json)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn sync(&self, auto: bool) -> Result<SyncDataResponse, String> {
        // Fetch profile
        log::info!("[sync::process] Fetching profile...");
        let profile = self.get_profile().await.map_err(|e| e.to_string())?;

        if !profile.success {
            return Err("Failed to fetch profile".to_string());
        }

        log::info!("[sync::process] Syncing profile...");

        let account = profile.response.account;
        let total_followers = account.follow_count;
        let total_subscribers = account.subscriber_count;

        log::info!("[sync::process] Account ID: {}, Followers: {}, Subscribers: {}",
            account.id, total_followers, total_subscribers);

        let mut followers: Vec<FanslyFollowersResponse> = Vec::new();
        let mut subscribers: Vec<Subscription> = Vec::new();

        log::info!("[sync::process] Fetching followers...");

        // Fetch followers until we have all of them
        let mut offset = 0;
        let mut total_requests = 0;
        while followers.len() < total_followers as usize {
            log::info!(
                "[sync::process] Fetching followers for account {} with offset {} (total: {})",
                account.id, offset, total_followers
            );
            let response = self
                .fetch_followers(&account.id, &self.token.as_ref().unwrap(), offset)
                .await
                .map_err(|e| e.to_string())?;

            log::info!(
                "[sync::process] Got {} followers from API.",
                response.response.len()
            );
            followers.extend(response.response);
            offset += 100;
            total_requests += 1;

            // Every 10 requests, sleep for a bit to avoid rate limiting
            if total_requests % 10 == 0 {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }

        // Fetch subscribers until we have all of them
        offset = 0;
        while subscribers.len() < total_subscribers as usize {
            log::info!(
                "[sync::process] Fetching subscribers with offset {} for account {} (total: {})",
                offset, account.id, total_subscribers
            );

            let response = self
                .fetch_subscribers(&self.token.as_ref().unwrap(), offset)
                .await
                .map_err(|e| e.to_string())?;

            subscribers.extend(response);
            offset += 100;
            total_requests += 1;

            // Every 10 requests, sleep for a bit to avoid rate limiting
            if total_requests % 10 == 0 {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }

        log::info!(
            "[sync::process] Got {} followers and {} subscribers from API.",
            followers.len(),
            subscribers.len()
        );

        log::info!("[sync::process] Sync complete.");
        log::info!("[sync::process] Uploading sync data to paste.hep.gg for processing...");

        // Upload sync data to paste.hep.gg
        if !auto {
            let paste_url = self
                .upload_sync_data(SyncDataResponse {
                    followers: followers.clone(),
                    subscribers: subscribers.clone(),
                    sync_data_url: "".to_string(),
                })
                .await
                .map_err(|e| e.to_string())?;

            // Return JSON of what we fetched
            Ok(SyncDataResponse {
                followers,
                subscribers,
                sync_data_url: paste_url,
            })
        } else {
            // Return JSON of what we fetched
            Ok(SyncDataResponse {
                followers,
                subscribers,
                sync_data_url: "".to_string(),
            })
        }
    }
}
