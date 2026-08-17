use std::{f32::consts::E, io::Write, path::PathBuf};

use anyhow::{Result, anyhow};
use directories::{self, ProjectDirs};
use iroh::SecretKey;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug)]
pub struct Setup {
    projdir: ProjectDirs,
    config: Config,
}

#[derive(Debug, Deserialize, Serialize)]
struct PublishKey {
    secret: SecretKey,
}

impl Setup {
    pub fn new(username: String) -> Result<Self> {
        let projdir = Self::get_dirs()?;
        // Create the config folder
        let fol = projdir.config_dir();
        match std::fs::create_dir(&fol) {
            Ok(_) => {}
            Err(e) => println!("{:#?}", e),
        }
        // Create the emptyish config file
        info!("Make Config");
        let mut config_path = fol.to_path_buf();
        config_path.push("config.toml");

        let config = match std::fs::exists(&config_path) {
            Ok(val) => {
                if val {
                    Config::load(config_path.clone())?
                } else {
                    let config = Config::new(username);
                    let _ = config.save(config_path);
                    config
                }
            }
            Err(e) => {
                error!("Error {}", e);
                return Err(anyhow!("{:#?}", e));
            }
        };

        info!("Make Key");
        // Create the author key if it does not exist
        let mut publish_key_path = projdir.config_dir().to_path_buf();
        publish_key_path.push("author.key");

        match std::fs::exists(&publish_key_path) {
            Ok(val) => {
                /* All good, do nothing */
                if val {
                    info!("key exists {}", val);
                } else {
                    //Create a new key
                    let publish_key = SecretKey::generate();
                    let val = PublishKey {
                        secret: publish_key,
                    };
                    let contents = toml::to_string(&val)?;
                    std::fs::write(publish_key_path, contents).expect("Can't write key file");
                };
            }
            Err(e) => {
                error!("Error {}", e);
            }
        }

        // Create the setup
        Ok(Self { projdir, config })
    }

    pub fn get_dirs() -> Result<ProjectDirs> {
        if let Some(conf_dir) = directories::ProjectDirs::from("com", "", env!("CARGO_PKG_NAME")) {
            Ok(conf_dir)
        } else {
            return Err(anyhow!("Can't find config dir"));
        }
    }

    pub fn get_author_secret(&self) -> Result<SecretKey> {
        let mut path = self.projdir.config_dir().to_path_buf();
        path.push("author.key");
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let the_key: PublishKey = toml::from_str(&content).expect("Bad config file");
                return Ok(the_key.secret);
            }
            Err(e) => {
                error!("Author Key fail {:}", e);
                return Err(anyhow!("Author Key Fail {:}", e));
            }
        }
    }

    pub fn config_path(&self) -> PathBuf {
        let path = self.projdir.config_dir().to_path_buf();
        return path;
    }

    pub fn database_path(&self) -> PathBuf {
        let mut path = self.projdir.config_dir().to_path_buf();
        path.push("database.db");
        return path;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    username: String,
}

impl Config {
    pub fn new(username: String) -> Self {
        Self { username }
    }

    pub fn save(&self, path: PathBuf) -> Result<()> {
        let contents = toml::to_string(&self).expect("Broken Config");
        std::fs::write(path, contents).expect("Can't write config file");
        Ok(())
    }

    pub fn load(path: PathBuf) -> Result<Self> {
        let config = match std::fs::read_to_string(path) {
            Ok(content) => {
                let content = content.as_str();
                let config: Config = toml::from_str(&content).expect("Bad config file");
                config
            }
            Err(e) => {
                return Err(anyhow!("Bad Config File Parse {:#?}", e));
            }
        };
        Ok(config)
    }
}
