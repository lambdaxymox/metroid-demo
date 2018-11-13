use std::fs::File;
use std::collections::HashMap;


#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Address {
    pub row: usize,
    pub column: usize,
}

impl Address {
    fn new(row: usize, column: usize) -> Self {
        Self {
            row: row, column: column
        }
    }
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
}

pub fn load(file: &str) -> Result<FontAtlas, Error> {
    let data = match File::open(file) {
        Ok(file) => file,
        Err(_) => return Err(Error::FileNotFound(file.to_owned())),
    };
    let font_atlas = match serde_json::from_reader(data) {
        Ok(val) => val,
        Err(_) => return Err(Error::CouldNotParseFontFile(file.to_owned())),
    };

    Ok(font_atlas)
}
