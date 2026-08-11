use std::path::PathBuf;

use anyhow::{Result, anyhow};
use directories::{self, ProjectDirs};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Setup {
    projdir: ProjectDirs,
    config: Config,
}

impl Setup {
    pub fn new(username: String) -> Result<Self> {
        let projdir = Self::get_path()?;
        // Create the config folder
        let fol = projdir.config_dir();
        match std::fs::create_dir(&fol) {
            Ok(_) => {}
            Err(e) => println!("{:#?}", e),
        }
        // Create the emptyish config file
        let config = Config::new(username);
        let mut path = fol.to_path_buf();
        path.push("config.toml");
        let _ = config.save(path);
        Ok(Self { projdir, config })
    }

    pub fn get_path() -> Result<ProjectDirs> {
        if let Some(conf_dir) = directories::ProjectDirs::from("com", "", env!("CARGO_PKG_NAME")) {
            Ok(conf_dir)
        } else {
            return Err(anyhow!("Can't find config dir"));
        }
    }

    pub fn config_path(&self) -> PathBuf { 
        let mut path = self.projdir.config_dir().to_path_buf();
        return path
    }

    pub fn database_path(&self) -> PathBuf { 
        let mut path = self.projdir.config_dir().to_path_buf();
        path.push("database.db");
        return path
    }


    pub fn open() -> Result<Self> {
        let projdir = Self::get_path()?;
        let mut path = projdir.config_dir().to_path_buf();
        path.push("config.toml");
        let config = Config::load(path)?;
        Ok(Self { projdir, config })
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
                return Err(anyhow!("Bad Config File Parse {:#?}",e));
            }
        };
        Ok(config)
    }
}
