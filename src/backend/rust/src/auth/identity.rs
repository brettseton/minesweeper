use crate::model::UserInfo;
use actix_identity::Identity;

pub trait IdentityExt {
    fn user_info(&self) -> Option<UserInfo>;
}

impl IdentityExt for Identity {
    fn user_info(&self) -> Option<UserInfo> {
        self.id()
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
    }
}
