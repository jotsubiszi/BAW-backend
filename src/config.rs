#[derive(Clone)]
pub struct Config {
    pub access_secret: String,
    pub refresh_secret: String,
    pub access_token_expiry_secs: i64,
    pub refresh_token_expiry_secs: i64,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            access_secret: std::env::var("ACCESS_SECRET").expect("ACCESS_SECRET not set"),
            refresh_secret: std::env::var("REFRESH_SECRET").expect("REFRESH_SECRET not set"),
            access_token_expiry_secs: std::env::var("ACCESS_EXPIRY_SECS")
                .unwrap_or_else(|_| "900".into())
                .parse()
                .unwrap(),
            refresh_token_expiry_secs: std::env::var("REFRESH_EXPIRY_SECS")
                .unwrap_or_else(|_| "604800".into())
                .parse()
                .unwrap(),
        }
    }
}
