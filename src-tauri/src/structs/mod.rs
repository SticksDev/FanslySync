use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncDataResponse {
    pub followers: Vec<FanslyFollowersResponse>,
    pub subscribers: Vec<Subscription>,
    pub sync_data_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FanslyBaseResponse<T> {
    pub success: bool,
    pub response: T,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FanslyBaseResponseList<T> {
    pub success: bool,
    pub response: Vec<T>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FanslyFollowersResponse {
    pub follower_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FanslySubscriptionsResponse {
    pub stats: SubscriptionsStats,
    pub subscriptions: Vec<Subscription>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionsStats {
    pub total_active: i64,
    pub total_expired: i64,
    pub total: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: String,
    pub history_id: String,
    pub subscriber_id: String,
    pub subscription_tier_id: String,
    pub subscription_tier_name: String,
    pub subscription_tier_color: String,
    pub plan_id: String,
    pub promo_id: Option<String>,
    pub gift_code_id: Value,
    pub payment_method_id: String,
    pub status: i64,
    pub price: i64,
    pub renew_price: i64,
    pub renew_correlation_id: String,
    pub auto_renew: i64,
    pub billing_cycle: i64,
    pub duration: i64,
    pub renew_date: i64,
    pub version: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub ends_at: i64,
    pub promo_price: Value,
    pub promo_duration: Value,
    pub promo_status: Value,
    pub promo_starts_at: Value,
    pub promo_ends_at: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FanslyAccountResponse {
    pub account: Account,
    pub correlation_id: String,
    pub check_token: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub flags: i64,
    pub version: i64,
    pub created_at: i64,
    pub follow_count: i64,
    pub subscriber_count: i64,
    pub permissions: Permissions,
    pub timeline_stats: TimelineStats,
    pub profile_access_flags: i64,
    pub profile_flags: i64,
    pub about: String,
    pub location: String,
    pub profile_socials: Vec<Value>,
    pub status_id: i64,
    pub last_seen_at: i64,
    pub post_likes: i64,
    pub streaming: Streaming,
    pub account_media_likes: i64,
    pub earnings_wallet: EarningsWallet,
    pub subscription_tiers: Vec<SubscriptionTier>,
    pub profile_access: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    pub account_permission_flags: AccountPermissionFlags,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountPermissionFlags {
    pub flags: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineStats {
    pub account_id: String,
    pub image_count: i64,
    pub video_count: i64,
    pub bundle_count: i64,
    pub bundle_image_count: i64,
    pub bundle_video_count: i64,
    pub fetched_at: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MainWallet {
    pub id: String,
    pub account_id: String,
    pub balance: i64,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub wallet_version: i64,
    pub flags: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Streaming {
    pub account_id: String,
    pub channel: Value,
    pub enabled: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsWallet {
    pub id: String,
    pub account_id: String,
    pub balance: i64,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub wallet_version: i64,
    pub flags: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionTier {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub color: String,
    pub pos: i64,
    pub price: i64,
    pub max_subscribers: i64,
    pub subscription_benefits: Vec<String>,
    pub included_tier_ids: Vec<Value>,
    pub plans: Vec<Plan>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub id: String,
    pub status: i64,
    pub billing_cycle: i64,
    pub price: i64,
    pub use_amounts: i64,
    pub promos: Vec<Value>,
    pub uses: i64,
}
