use std::path::PathBuf;

use anyhow::{Result, anyhow};
use directories::{self, ProjectDirs};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Setup {
    config_path: ProjectDirs,
}

impl Setup {
    pub fn new(username: String) -> Result<Self> {
        let config_path = Self::get_path()?;
        // Create the config folder
        let fol = config_path.config_dir();
        match std::fs::create_dir(&fol) {
            Ok(_) => {},
            Err(e) => println!("{:#?}",e),
        }
        // Create the emptyish config file 
        let conf = Config::new(username);
        let mut path = fol.to_path_buf();
        path.push("config.toml");
        println!("{:#?}",path);
        let _ = conf.save(path);
        Ok(Self { config_path })
    }

    pub fn get_path() -> Result<ProjectDirs> {
        if let Some(conf_dir) = directories::ProjectDirs::from("com", "", env!("CARGO_PKG_NAME")) {
            Ok(conf_dir)
        } else {
            return Err(anyhow!("Can't find config dir"));
        }
    }

    pub fn open() -> Result<Self> {
        let config_path = Self::get_path()?;        
        Ok(Self{ config_path})
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    username: String,
}

impl Config { 
    pub fn new(username: String ) -> Self { 
        Self { 
            username
        }
    }

    pub fn save(&self,path: PathBuf) ->  Result<()>{ 
        let contents = toml::to_string(&self).expect("Broken Config");
        std::fs::write(path,contents).expect("Can't write config file");
        Ok(())
    }
}