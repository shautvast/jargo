use std::path::PathBuf;
use std::sync::OnceLock;

/// Contains any config elements
pub struct Config {
    pub cache_location: String,
    pub user_home: String,
}

pub static CONFIG: OnceLock<Config> = OnceLock::new();

/// default config
pub fn config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let user_home = home::home_dir().map(|p|p.to_str().unwrap().to_owned())
            .expect("Can not find $HOME in environment");
        Config {
            cache_location: format!("{}/jargo/repo", user_home).into(), //TODO make it '.jargo'
            user_home,
        }
    });
    CONFIG.get().unwrap()
}
