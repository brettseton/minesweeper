use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub sub: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserStatsDto {
    pub won: i32,
    pub lost: i32,
    pub in_progress: i32,
}
