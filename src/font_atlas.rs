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

pub fn load(file: &str) -> FontAtlas {
    let data = File::open(file).expect("File not found.");
    let font_atlas = serde_json::from_reader(data).unwrap();

    font_atlas
}
