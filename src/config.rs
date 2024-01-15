use std::path::PathBuf;
use std::sync::OnceLock;

pub struct Config {
    pub cache_location: String,
    pub maven_central: String,
    pub user_home: PathBuf,
}

pub static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let user_home = home::home_dir().unwrap();
        Config {
            cache_location: format!("{}/jargo/repo", user_home.to_str().unwrap()).into(),
            user_home,
            maven_central: "https://repo.maven.apache.org/maven2".into()
        }
    });
    CONFIG.get().unwrap()
}