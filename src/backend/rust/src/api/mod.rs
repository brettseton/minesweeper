pub mod auth;
pub mod game;
pub mod user;

pub use auth::config as config_auth;
pub use game::config as config_game;
pub use user::config as config_user;

pub use auth::SCOPE_ACCOUNT;
pub use game::SCOPE_GAME;
pub use user::SCOPE_USER;

pub use auth::{PATH_CALLBACK, PATH_LOGIN, PATH_LOGOUT, PATH_STATUS};
pub use game::{PATH_FLAG_ID, PATH_ID, PATH_NEW, PATH_NEW_CUSTOM};
pub use user::{PATH_GAMES, PATH_STATS};
