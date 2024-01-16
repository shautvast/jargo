use std::path::PathBuf;
use std::sync::OnceLock;

/// Contains any config elements
pub struct Config {
    pub cache_location: String,
    pub user_home: PathBuf,
}

pub static CONFIG: OnceLock<Config> = OnceLock::new();

/// default config
pub fn config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let user_home = home::home_dir().unwrap();
        Config {
            cache_location: format!("{}/jargo/repo", user_home.to_str().unwrap()).into(), //TODO make it '.jargo'
            user_home,
        }
    });
    CONFIG.get().unwrap()
}
