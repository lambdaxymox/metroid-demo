use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use toml;


#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    pub gl_log_file: String,
    pub skybox_panel: PathBuf,
    pub ground_plane_tex: PathBuf,
    pub text_font_sheet: PathBuf,
    pub title_font_sheet: PathBuf,
    pub shader_path: PathBuf,
    pub shader_version: PathBuf,
    pub asset_path: PathBuf,
}

#[derive(Clone, Deserialize, Serialize)]
struct Platform {
    macos: MacOS,
    windows: Windows,
    linux: Linux,
}

#[derive(Clone, Deserialize, Serialize)]
struct MacOS {
    shader_version: PathBuf
}

#[derive(Clone, Deserialize, Serialize)]
struct Windows {
    shader_version: PathBuf,
}

#[derive(Clone, Deserialize, Serialize)]
struct Linux {
    shader_version: PathBuf,
}

#[derive(Clone, Debug)]
pub enum Error {
    ConfigFileNotFound(String),
    CouldNotReadConfig(String),
    Deserialize(toml::de::Error),
}

fn get_content<P: AsRef<Path>>(path: &P) -> Result<String, Error> {
    let mut file = match File::open(path) {
        Ok(val) => val,
        Err(_) => {
            return Err(Error::ConfigFileNotFound(format!("{}", path.as_ref().display())));
        }
    };

    let mut content = String::new();
    let _bytes_read = match file.read_to_string(&mut content) {
        Ok(val) => val,
        Err(_) => {
            return Err(Error::CouldNotReadConfig(format!("{}", path.as_ref().display())));
        }
    };

    Ok(content)
}

pub fn load<P: AsRef<Path>>(path: P) -> Result<Config, Error> {
    let content = get_content(&path)?;
    match toml::from_str(&content) {
        Ok(config) => Ok(config),
        Err(e) => Err(Error::Deserialize(e)),
    }
}
