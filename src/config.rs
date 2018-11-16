use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;


#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    pub gl_log_file: PathBuf,
    pub cube_map_front: PathBuf,
    pub cube_map_back: PathBuf,
    pub cube_map_left: PathBuf,
    pub cube_map_right: PathBuf,
    pub cube_map_top: PathBuf,
    pub cube_map_bottom: PathBuf,
    pub ground_plane_tex: PathBuf,
    pub text_font_sheet: PathBuf,
    pub title_font_sheet: PathBuf,
    pub shader_path: PathBuf,
    pub asset_path: PathBuf,
}

fn get_content<P: AsRef<Path>>(path: &P) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut content = String::new();
    file.read_to_string(&mut content).ok()?;

    Some(content)
}

pub fn load<P: AsRef<Path>>(path: P) -> Option<Config> {
    if let Some(content) = get_content(&path) {
        toml::from_str(&content).ok()
    } else {
        None
    }
}
