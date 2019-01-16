use std::fs::File;
use std::io;
use std::path::Path;
use std::collections::HashMap;


#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Address {
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FontAtlas {
    pub glyph_y_offsets: HashMap<char, f32>,
    pub glyph_widths: HashMap<char, f32>,
    pub glyph_coords: HashMap<char, Address>,
    pub rows: usize,
    pub columns: usize,
}

#[derive(Debug, Clone)]
pub enum Error {
    FileNotFound(String),
    CouldNotParseFontFile(String),
    CouldNotParseBuffer,
}

pub fn load_reader<R: io::Read>(reader: R) -> Result<FontAtlas, Error> {
    let font_atlas = serde_json::from_reader(reader).map_err(|_e| {
        Error::CouldNotParseBuffer
    })?;

    Ok(font_atlas)
}

pub fn load_file<P: AsRef<Path>>(file: P) -> Result<FontAtlas, Error> {
    let data = match File::open(file.as_ref()) {
        Ok(handle) => handle,
        Err(_) => {
            return Err(
                Error::FileNotFound(format!("{}", file.as_ref().display()))
            );
        }
    };
    let font_atlas = match load_reader(data) {
        Ok(val) => val,
        Err(_) => {
            return Err(
                Error::CouldNotParseFontFile(format!("{}", file.as_ref().display()))
            );
        }
    };

    Ok(font_atlas)
}
