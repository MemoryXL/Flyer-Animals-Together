use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_PRIOR, VK_NEXT};

const CONFIG_DIR: &str = "Flyer-Animals-Together";
const CONFIG_FILE: &str = "config.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub key_y_velocity_up: i32,
    pub key_y_velocity_down: i32,
    pub y_velocity_up_val: f32,
    pub y_velocity_down_val: f32,
    pub notifications_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            key_y_velocity_up: VK_PRIOR.0 as i32,
            key_y_velocity_down: VK_NEXT.0 as i32,
            y_velocity_up_val: 10.0,
            y_velocity_down_val: -8.0,
            notifications_enabled: true,
        }
    }
}

impl Config {
    fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let appdata = std::env::var("APPDATA")?;
        let config_dir = PathBuf::from(appdata).join(CONFIG_DIR);
        Ok(config_dir.join(CONFIG_FILE))
    }

    pub fn load() -> Self {
        match Self::get_config_path() {
            Ok(path) => {
                if path.exists() {
                    match fs::read_to_string(&path) {
                        Ok(content) => {
                            match serde_json::from_str::<Config>(&content) {
                                Ok(config) => config,
                                Err(e) => {
                                    eprintln!("Failed to parse config: {}, using defaults", e);
                                    let default = Self::default();
                                    let _ = default.save();
                                    default
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read config file: {}, using defaults", e);
                            let default = Self::default();
                            let _ = default.save();
                            default
                        }
                    }
                } else {
                    let default = Self::default();
                    let _ = default.save();
                    default
                }
            }
            Err(e) => {
                eprintln!("Failed to get config path: {}, using defaults", e);
                Self::default()
            }
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_config_path()?;
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        
        Ok(())
    }
}