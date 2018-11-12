extern crate glfw;
extern crate chrono;
extern crate stb_image;
extern crate simple_cgmath;

#[macro_use]
mod logger;

mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

mod gl_helpers;
mod camera;

use glfw::{Action, Context, Key};
use gl::types::{GLenum, GLfloat, GLint, GLsizeiptr, GLvoid, GLuint};

use stb_image::image;
use stb_image::image::LoadResult;

use std::collections::HashMap;
use std::mem;
use std::ptr;
use std::process;

use gl_helpers as glh;
use simple_cgmath as math;
use math::{Matrix4, Quaternion, AsArray};
use camera::Camera;

// OpenGL extension constants.
const GL_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;
const GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;

// Log file.
const GL_LOG_FILE: &str = "gl.log";

// Textures.
const CUBE_MAP: &str = "skybox_panel.png";
const FRONT: &str = CUBE_MAP;
const BACK: &str = CUBE_MAP;
const LEFT: &str = CUBE_MAP;
const RIGHT: &str = CUBE_MAP;
const TOP: &str = CUBE_MAP;
const BOTTOM: &str = CUBE_MAP;
const GROUND_PLANE_TEX: &str = "tile_rock_planet256x256.png";
const TEXT_FONT_SHEET: &str = "font2048x2048.png";
const TITLE_FONT_SHEET: &str = "title_font2048x2048.png";

// Text colors.
const TITLE_COLOR: [f32; 3] = [1.0, 1.0, 1.0];
const TEXT_COLOR: [f32; 3] = [139 as f32 / 255 as f32, 193 as f32 / 255 as f32, 248 as f32 / 255 as f32];

// Shader paths.
#[cfg(target_os = "macos")]
const SHADER_PATH: &str = "shaders/330";

#[cfg(target_os = "windows")]
const SHADER_PATH: &str = "shaders/330";

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const SHADER_PATH: &str = "shaders/420";

const ASSET_PATH: &str = "assets";


fn shader_file(file: &str) -> String {
    format!("{}/{}", SHADER_PATH, file)
}

fn asset_file(file: &str) -> String {
    format!("{}/{}", ASSET_PATH, file)
}

struct FontAtlas {
    glyph_y_offsets: HashMap<char, f32>,
    glyph_widths: HashMap<char, f32>,
    coords: HashMap<char, (usize, usize)>,
    bitmap: Vec<u8>,
    rows: usize,
    columns: usize,
}

fn load_text_font_atlas() -> FontAtlas {
    let coords = [
        (' ', (0, 0)),
        ('A', (1, 1)), ('B', (1, 2)), ('C', (1, 3)), ('D', (1, 4)), ('E', (1, 5)), ('F', (1, 6)),
        ('G', (2, 1)), ('H', (2, 2)), ('I', (2, 3)), ('J', (2, 4)), ('K', (2, 5)), ('L', (2, 6)),
        ('M', (3, 1)), ('N', (3, 2)), ('O', (3, 3)), ('P', (3, 4)), ('Q', (3, 5)), ('R', (3, 6)),
        ('S', (4, 1)), ('T', (4, 2)), ('U', (4, 3)), ('V', (4, 4)), ('W', (4, 5)), ('X', (4, 6)),
        ('Y', (5, 1)), ('Z', (5, 2)), ('0', (5, 3)), ('1', (5, 4)), ('2', (5, 5)), ('3', (5, 6)),
        ('4', (6, 1)), ('5', (6, 2)), ('6', (6, 3)), ('7', (6, 4)), ('8', (6, 5)), ('9', (6, 6)),
        ('a', (1, 1)), ('b', (1, 2)), ('c', (1, 3)), ('d', (1, 4)), ('e', (1, 5)), ('f', (1, 6)),
        ('g', (2, 1)), ('h', (2, 2)), ('i', (2, 3)), ('j', (2, 4)), ('k', (2, 5)), ('l', (2, 6)),
        ('m', (3, 1)), ('n', (3, 2)), ('o', (3, 3)), ('p', (3, 4)), ('q', (3, 5)), ('r', (3, 6)),
        ('s', (4, 1)), ('t', (4, 2)), ('u', (4, 3)), ('v', (4, 4)), ('w', (4, 5)), ('x', (4, 6)),
        ('y', (5, 1)), ('z', (5, 2)), 
    ].iter().cloned().collect();
    let glyph_y_offsets = [
        (' ', 0.0),
        ('A', 0.0), ('B', 0.0), ('C', 0.0), ('D', 0.0), ('E', 0.0), ('F', 0.0),
        ('G', 0.0), ('H', 0.0), ('I', 0.0), ('J', 0.0), ('K', 0.0), ('L', 0.0),
        ('M', 0.0), ('N', 0.0), ('O', 0.0), ('P', 0.0), ('Q', 0.0), ('R', 0.0),
        ('S', 0.0), ('T', 0.0), ('U', 0.0), ('V', 0.0), ('W', 0.0), ('X', 0.0),
        ('Y', 0.0), ('Z', 0.0), ('0', 0.0), ('1', 0.0), ('2', 0.0), ('3', 0.0),
        ('4', 0.0), ('5', 0.0), ('6', 0.0), ('7', 0.0), ('8', 0.0), ('9', 0.0),
        ('g', 0.0), ('h', 0.0), ('i', 0.0), ('j', 0.0), ('k', 0.0), ('l', 0.0),
        ('a', 0.0), ('b', 0.0), ('c', 0.0), ('d', 0.0), ('e', 0.0), ('f', 0.0),
        ('m', 0.0), ('n', 0.0), ('o', 0.0), ('p', 0.0), ('q', 0.0), ('r', 0.0),
        ('s', 0.0), ('t', 0.0), ('u', 0.0), ('v', 0.0), ('w', 0.0), ('x', 0.0),
        ('y', 0.0), ('z', 0.0),        
    ].iter().cloned().collect();
    let glyph_widths = [
        (' ', 1.0),
        ('A', 1.0), ('B', 1.0), ('C', 1.0), ('D', 1.0), ('E', 1.0), ('F', 1.0),
        ('G', 1.0), ('H', 1.0), ('I', 1.0), ('J', 1.0), ('K', 1.0), ('L', 1.0),
        ('M', 1.0), ('N', 1.0), ('O', 1.0), ('P', 1.0), ('Q', 1.0), ('R', 1.0),
        ('S', 1.0), ('T', 1.0), ('U', 1.0), ('V', 1.0), ('W', 1.0), ('X', 1.0),
        ('Y', 1.0), ('Z', 1.0), ('0', 1.0), ('1', 1.0), ('2', 1.0), ('3', 1.0),
        ('4', 1.0), ('5', 1.0), ('6', 1.0), ('7', 1.0), ('8', 1.0), ('9', 1.0),
        ('g', 1.0), ('h', 1.0), ('i', 1.0), ('j', 1.0), ('k', 1.0), ('l', 1.0),
        ('a', 1.0), ('b', 1.0), ('c', 1.0), ('d', 1.0), ('e', 1.0), ('f', 1.0),
        ('m', 1.0), ('n', 1.0), ('o', 1.0), ('p', 1.0), ('q', 1.0), ('r', 1.0),
        ('s', 1.0), ('t', 1.0), ('u', 1.0), ('v', 1.0), ('w', 1.0), ('x', 1.0),
        ('y', 1.0), ('z', 1.0), 
    ].iter().cloned().collect();
    let bitmap = vec![];
    let rows = 8;
    let columns = 8;

    FontAtlas {
        glyph_y_offsets: glyph_y_offsets,
        glyph_widths: glyph_widths,
        coords: coords,
        bitmap: bitmap,
        rows: rows,
        columns: columns,
    }
}

fn load_title_font_atlas() -> FontAtlas {
    let coords = [
        (' ', (0,  0)), ('A', (0,  1)), ('B', (0,  2)), ('C', (0,  3)),
        ('D', (0,  4)), ('E', (0,  5)), ('F', (0,  6)), ('G', (0,  7)),
        ('H', (1,  0)), ('I', (1,  1)), ('J', (1,  2)), ('K', (1,  3)),
        ('L', (1,  4)), ('M', (1,  5)), ('N', (1,  6)), ('O', (1,  7)),
        ('P', (2,  0)), ('Q', (2,  1)), ('R', (2,  2)), ('S', (2,  3)),
        ('T', (2,  4)), ('U', (2,  5)), ('V', (2,  6)), ('W', (2,  7)),
        ('X', (3,  0)), ('Y', (3,  1)), ('Z', (3,  2)),
                        ('a', (0,  1)), ('b', (0,  2)), ('c', (0,  3)),
        ('d', (0,  4)), ('e', (0,  5)), ('f', (0,  6)), ('g', (0,  7)),
        ('h', (1,  0)), ('i', (1,  1)), ('j', (1,  2)), ('k', (1,  3)),
        ('l', (1,  4)), ('m', (1,  5)), ('n', (1,  6)), ('o', (1,  7)),
        ('p', (2,  0)), ('q', (2,  1)), ('r', (2,  2)), ('s', (2,  3)),
        ('t', (2,  4)), ('u', (2,  5)), ('v', (2,  6)), ('w', (2,  7)),
        ('x', (3,  0)), ('y', (3,  1)), ('z', (3,  2)),
    ].iter().cloned().collect();
    let glyph_y_offsets = [
        (' ', 0.0), ('A', 0.0), ('B', 0.0), ('C', 0.0),
        ('D', 0.0), ('E', 0.0), ('F', 0.0), ('G', 0.0),
        ('H', 0.0), ('I', 0.0), ('J', 0.0), ('K', 0.0),
        ('L', 0.0), ('M', 0.0), ('N', 0.0), ('O', 0.0),
        ('P', 0.0), ('Q', 0.0), ('R', 0.0), ('S', 0.0),
        ('T', 0.0), ('U', 0.0), ('V', 0.0), ('W', 0.0),
        ('X', 0.0), ('Y', 0.0), ('Z', 0.0),
                    ('a', 0.0), ('b', 0.0), ('c', 0.0),
        ('d', 0.0), ('e', 0.0), ('f', 0.0), ('g', 0.0),
        ('h', 0.0), ('i', 0.0), ('j', 0.0), ('k', 0.0),
        ('l', 0.0), ('m', 0.0), ('n', 0.0), ('o', 0.0),
        ('p', 0.0), ('q', 0.0), ('r', 0.0), ('s', 0.0),
        ('t', 0.0), ('u', 0.0), ('v', 0.0), ('w', 0.0),
        ('x', 0.0), ('y', 0.0), ('z', 0.0),
    ].iter().cloned().collect();
    let glyph_widths = [
        (' ', 0.40), ('A', 0.40), ('B', 0.40), ('C', 0.40),
        ('D', 0.40), ('E', 0.30), ('F', 0.30), ('G', 0.40),
        ('H', 0.40), ('I', 0.25), ('J', 0.40), ('K', 0.40),
        ('L', 0.40), ('M', 0.50), ('N', 0.50), ('O', 0.40),
        ('P', 0.40), ('Q', 0.40), ('R', 0.40), ('S', 0.40),
        ('T', 0.40), ('U', 0.40), ('V', 0.40), ('W', 0.50),
        ('X', 0.40), ('Y', 0.40), ('Z', 0.40),
                     ('a', 0.40), ('b', 0.40), ('c', 0.40),
        ('d', 0.40), ('e', 0.30), ('f', 0.30), ('g', 0.40),
        ('h', 0.40), ('i', 0.25), ('j', 0.40), ('k', 0.40),
        ('l', 0.40), ('m', 0.50), ('n', 0.50), ('o', 0.40),
        ('p', 0.40), ('q', 0.40), ('r', 0.40), ('s', 0.40),
        ('t', 0.40), ('u', 0.40), ('v', 0.40), ('w', 0.50),
        ('x', 0.40), ('y', 0.40), ('z', 0.40),
    ].iter().cloned().collect();
    let bitmap = vec![];
    let rows = 8;
    let columns = 8;

    FontAtlas {
        glyph_y_offsets: glyph_y_offsets,
        glyph_widths: glyph_widths,
        coords: coords,
        bitmap: bitmap,
        rows: rows,
        columns: columns,
    }
}

///
/// Create the shaders for rendering text.
///
fn create_title_screen_shaders(context: &glh::GLContext) -> (GLuint, GLint) {
    let title_screen_sp = glh::create_program_from_files(
        context, &shader_file("title_screen.vert.glsl"), &shader_file("title_screen.frag.glsl")
    );
    assert!(title_screen_sp > 0);

    let title_screen_sp_text_color_loc = unsafe { 
        gl::GetUniformLocation(title_screen_sp, glh::gl_str("text_color").as_ptr())
    };
    assert!(title_screen_sp_text_color_loc > -1);

    (title_screen_sp, title_screen_sp_text_color_loc)
}

///
/// Set up the geometry for rendering title screen text.
///
fn create_title_screen_geometry(
    context: &glh::GLContext, 
    font_atlas: &FontAtlas, text: &str,
    x_pos: f32, y_pos: f32, pixel_scale: f32) -> (GLuint, GLuint, GLuint, usize) {
    
    let mut string_vp_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut string_vp_vbo);
    }

    let mut string_vt_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut string_vt_vbo);
    }

    let mut string_point_count = 0;
    text_to_vbo(
        &context, text, &font_atlas, 
        x_pos, y_pos, pixel_scale, 
        &mut string_vp_vbo, &mut string_vt_vbo, &mut string_point_count
    );

    let mut string_vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut string_vao);
        gl::BindVertexArray(string_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, string_vp_vbo);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, string_vt_vbo);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(1);
    }
    assert!(string_vao > 0);

    (string_vp_vbo, string_vt_vbo, string_vao, string_point_count)
}

///
/// Print a string to the GLFW screen with the given font.
///
fn text_to_vbo(
    context: &glh::GLContext, st: &str, atlas: &FontAtlas,
    start_x: f32, start_y: f32, scale_px: f32,
    points_vbo: &mut GLuint, texcoords_vbo: &mut GLuint, point_count: &mut usize) {

    let mut points_temp = vec![0.0; 12 * st.len()];
    let mut texcoords_temp = vec![0.0; 12 * st.len()];
    let mut at_x = start_x;
    let at_y = start_y;

    for (i, ch_i) in st.chars().enumerate() {
        let (atlas_row, atlas_col) = atlas.coords[&ch_i];
        
        let s = (atlas_col as f32) * (1.0 / (atlas.columns as f32));
        let t = ((atlas_row + 1) as f32) * (1.0 / (atlas.rows as f32));

        let x_pos = at_x;
        let y_pos = at_y - (scale_px / (context.height as f32)) * atlas.glyph_y_offsets[&ch_i];

        at_x +=  atlas.glyph_widths[&ch_i] * (scale_px / (context.width as f32));

        points_temp[12 * i]     = x_pos;
        points_temp[12 * i + 1] = y_pos;
        points_temp[12 * i + 2] = x_pos;
        points_temp[12 * i + 3] = y_pos - scale_px / (context.height as f32);
        points_temp[12 * i + 4] = x_pos + scale_px / (context.width as f32);
        points_temp[12 * i + 5] = y_pos - scale_px / (context.height as f32);

        points_temp[12 * i + 6]  = x_pos + scale_px / (context.width as f32);
        points_temp[12 * i + 7]  = y_pos - scale_px / (context.height as f32);
        points_temp[12 * i + 8]  = x_pos + scale_px / (context.width as f32);
        points_temp[12 * i + 9]  = y_pos;
        points_temp[12 * i + 10] = x_pos;
        points_temp[12 * i + 11] = y_pos;

        texcoords_temp[12 * i]     = s;
        texcoords_temp[12 * i + 1] = 1.0 - t + 1.0 / (atlas.rows as f32);
        texcoords_temp[12 * i + 2] = s;
        texcoords_temp[12 * i + 3] = 1.0 - t;
        texcoords_temp[12 * i + 4] = s + 1.0 / (atlas.columns as f32);
        texcoords_temp[12 * i + 5] = 1.0 - t;

        texcoords_temp[12 * i + 6]  = s + 1.0 / (atlas.columns as f32);
        texcoords_temp[12 * i + 7]  = 1.0 - t;
        texcoords_temp[12 * i + 8]  = s + 1.0 / (atlas.columns as f32);
        texcoords_temp[12 * i + 9]  = 1.0 - t + 1.0 / (atlas.rows as f32);
        texcoords_temp[12 * i + 10] = s;
        texcoords_temp[12 * i + 11] = 1.0 - t + 1.0 / (atlas.rows as f32);
    }

    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, *points_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (12 * st.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            points_temp.as_ptr() as *const GLvoid, gl::DYNAMIC_DRAW
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, *texcoords_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (12 * st.len() * mem::size_of::<GLfloat>()) as GLsizeiptr, 
            texcoords_temp.as_ptr() as *const GLvoid, gl::DYNAMIC_DRAW
        );
    }

    *point_count = 6 * st.len();
}

///
/// Load the vertex buffer object for the skybox.
///
fn create_cube_map_geometry() -> GLuint {
    let cube_map_points: [GLfloat; 108] = [
        -10.0,  10.0, -10.0, -10.0, -10.0, -10.0,  10.0, -10.0, -10.0,
         10.0, -10.0, -10.0,  10.0,  10.0, -10.0, -10.0,  10.0, -10.0,

        -10.0, -10.0,  10.0, -10.0, -10.0, -10.0, -10.0,  10.0, -10.0,
        -10.0,  10.0, -10.0, -10.0,  10.0,  10.0, -10.0, -10.0,  10.0,

         10.0, -10.0, -10.0,  10.0, -10.0,  10.0,  10.0,  10.0,  10.0,
         10.0,  10.0,  10.0,  10.0,  10.0, -10.0,  10.0, -10.0, -10.0,

        -10.0, -10.0,  10.0, -10.0,  10.0,  10.0,  10.0,  10.0,  10.0,
         10.0,  10.0,  10.0,  10.0, -10.0,  10.0, -10.0, -10.0,  10.0,

        -10.0,  10.0, -10.0,  10.0,  10.0, -10.0,  10.0,  10.0,  10.0,
         10.0,  10.0,  10.0, -10.0,  10.0,  10.0, -10.0,  10.0, -10.0,

        -10.0, -10.0, -10.0, -10.0, -10.0,  10.0,  10.0, -10.0, -10.0,
         10.0, -10.0, -10.0, -10.0, -10.0,  10.0,  10.0, -10.0,  10.0
    ];

    let mut cube_map_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut cube_map_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, cube_map_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (3 * 36 * mem::size_of::<GLfloat>()) as GLsizeiptr,
            cube_map_points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
    }

    let mut cube_map_vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut cube_map_vao);
        gl::BindVertexArray(cube_map_vao);
        gl::EnableVertexAttribArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, cube_map_vbo);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
    }

    cube_map_vao
}

///
/// Load one of the cube map sides into a cube map texture.
///
fn load_cube_map_side(texture: GLuint, side_target: GLenum, file_name: &str) -> bool {
    unsafe {
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture);
    }

    let force_channels = 4;
    let image_data = match image::load_with_depth(file_name, force_channels, false) {
        LoadResult::ImageU8(image_data) => image_data,
        LoadResult::Error(_) => {
            eprintln!("ERROR: could not load {}", file_name);
            return false;
        }
        LoadResult::ImageF32(_) => {
            eprintln!("ERROR: Tried to load an image as byte vectors, got f32: {}", file_name);
            return false;
        }
    };

    let width = image_data.width;
    let height = image_data.height;

    // Check that the image size is a power of two.
    if (width & (width - 1)) != 0 || (height & (height - 1)) != 0 {
        eprintln!("WARNING: Texture {} lacks dimensions that are a power of two", file_name);
    }

    // copy image data into 'target' side of cube map
    unsafe {
        gl::TexImage2D(
            side_target, 0, gl::RGBA as i32, width as i32, height as i32, 0, 
            gl::RGBA, gl::UNSIGNED_BYTE,
            image_data.data.as_ptr() as *const GLvoid
        );
    }

    true
}

///
/// Create a cube map texture. Load all 6 sides of a cube map from images, 
/// and then format texture.
///
fn create_cube_map(
    front: &str, back: &str, top: &str,
    bottom: &str, left: &str, right: &str, tex_cube: &mut GLuint) {
    
    // Generate a cube map texture.
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0);
        gl::GenTextures(1, tex_cube);
    }

    // Load each image and copy it into a side of the cube-map texture.
    load_cube_map_side(*tex_cube, gl::TEXTURE_CUBE_MAP_NEGATIVE_Z, front);
    load_cube_map_side(*tex_cube, gl::TEXTURE_CUBE_MAP_POSITIVE_Z, back);
    load_cube_map_side(*tex_cube, gl::TEXTURE_CUBE_MAP_POSITIVE_Y, top);
    load_cube_map_side(*tex_cube, gl::TEXTURE_CUBE_MAP_NEGATIVE_Y, bottom);
    load_cube_map_side(*tex_cube, gl::TEXTURE_CUBE_MAP_NEGATIVE_X, left);
    load_cube_map_side(*tex_cube, gl::TEXTURE_CUBE_MAP_POSITIVE_X, right);
    
    // Format the cube map texture.
    unsafe {
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    }
}

///
/// Create the cube map shaders.
///
fn create_cube_map_shaders(context: &glh::GLContext) -> (GLuint, GLint, GLint) {
    let cube_sp = glh::create_program_from_files(
        &context, &shader_file("cube.vert.glsl"), &shader_file("cube.frag.glsl")
    );
    assert!(cube_sp > 0);

    // NOTE: This view matrix should *NOT* contain camera translation.
    let cube_view_mat_location = unsafe {
        gl::GetUniformLocation(cube_sp, glh::gl_str("view").as_ptr())
    };
    assert!(cube_view_mat_location > -1);

    let cube_proj_mat_location = unsafe {
        gl::GetUniformLocation(cube_sp, glh::gl_str("proj").as_ptr())
    };
    assert!(cube_proj_mat_location > -1);

    (cube_sp, cube_view_mat_location, cube_proj_mat_location)
}

///
/// Create the ground plane shaders.
///
fn create_ground_plane_shaders(context: &glh::GLContext) -> (GLuint, GLint, GLint) {
    // Here I used negative y from the buffer as the z value so that it was on
    // the floor but also that the 'front' was on the top side. also note how I
    // work out the texture coordinates, st, from the vertex point position.
    let gp_sp = glh::create_program_from_files(
        context, &shader_file("ground_plane.vert.glsl"), &shader_file("ground_plane.frag.glsl")
    );
    assert!(gp_sp > 0);

    let gp_view_mat_loc = unsafe { 
        gl::GetUniformLocation(gp_sp, glh::gl_str("view").as_ptr())
    };
    assert!(gp_view_mat_loc > -1);

    let gp_proj_mat_loc = unsafe {
        gl::GetUniformLocation(gp_sp, glh::gl_str("proj").as_ptr())
    };
    assert!(gp_proj_mat_loc > -1);

    (gp_sp, gp_view_mat_loc, gp_proj_mat_loc)
}

///
/// Create the ground plane geometry.
///
#[allow(unused_variables)]
fn create_ground_plane_geometry(context: &glh::GLContext) -> (GLuint, GLuint) {
    let ground_plane_points: [GLfloat; 18] = [
         20.0,  10.0, 0.0, -20.0,  10.0, 0.0, -20.0, -10.0, 0.0, 
        -20.0, -10.0, 0.0,  20.0, -10.0, 0.0,  20.0,  10.0, 0.0
    ];

    let mut points_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut points_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::BufferData( 
            gl::ARRAY_BUFFER, (mem::size_of::<GLfloat>() * ground_plane_points.len()) as GLsizeiptr,
            ground_plane_points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
    }
    assert!(points_vbo > 0);

    let mut points_vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut points_vao);
        gl::BindVertexArray(points_vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(0);
    }
    assert!(points_vao > 0);

    (points_vbo, points_vao)
}

fn create_camera(width: u32, height: u32) -> Camera {
    let near = 0.1;
    let far = 100.0;
    let fov = 67.0;
    let aspect = width as f32 / height as f32;

    let cam_speed: GLfloat = 3.0;
    let cam_yaw_speed: GLfloat = 50.0;

    let fwd = math::vec4((0.0, 0.98, -0.19, 0.0));
    let rgt = math::vec4((1.0, 0.0,  0.0, 0.0));
    let up  = math::vec4((0.0, 0.22,  0.98, 0.0));
    let cam_pos = math::vec3((0.0, -6.81, 3.96));
    
    let axis = Quaternion::new(0.77, 0.64, 0.0, 0.0);

    Camera::new(near, far, fov, aspect, cam_speed, cam_yaw_speed, cam_pos, fwd, rgt, up, axis)
}

fn reset_camera_to_default(context: &glh::GLContext, camera: &mut Camera) {
    let width = context.width;
    let height = context.height;
    *camera = create_camera(width, height);
}

///
/// Load textures.
///
fn load_texture(file_name: &str, tex: &mut GLuint, wrapping_mode: GLuint) -> bool {
    let force_channels = 4;
    let mut image_data = match image::load_with_depth(file_name, force_channels, false) {
        LoadResult::ImageU8(image_data) => image_data,
        LoadResult::Error(_) => {
            eprintln!("ERROR: could not load {}", file_name);
            return false;
        }
        LoadResult::ImageF32(_) => {
            eprintln!("ERROR: Tried to load an image as byte vectors, got f32: {}", file_name);
            return false;
        }
    };

    let width = image_data.width;
    let height = image_data.height;

    // Check that the image size is a power of two.
    if (width & (width - 1)) != 0 || (height & (height - 1)) != 0 {
        eprintln!("WARNING: texture {} is not power-of-2 dimensions", file_name);
    }

    let width_in_bytes = 4 *width;
    let half_height = height / 2;
    for row in 0..half_height {
        for col in 0..width_in_bytes {
            let temp = image_data.data[row * width_in_bytes + col];
            image_data.data[row * width_in_bytes + col] = image_data.data[((height - row - 1) * width_in_bytes) + col];
            image_data.data[((height - row - 1) * width_in_bytes) + col] = temp;
        }
    }

    unsafe {
        gl::GenTextures(1, tex);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, *tex);
        gl::TexImage2D(
            gl::TEXTURE_2D, 0, gl::RGBA as i32, width as i32, height as i32, 0, 
            gl::RGBA, gl::UNSIGNED_BYTE, 
            image_data.data.as_ptr() as *const GLvoid
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrapping_mode as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrapping_mode as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as GLint);
    }

    let mut max_aniso = 0.0;
    unsafe {
        gl::GetFloatv(GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut max_aniso);
        // Set the maximum!
        gl::TexParameterf(gl::TEXTURE_2D, GL_TEXTURE_MAX_ANISOTROPY_EXT, max_aniso);
    }

    true
}

///
/// The GLFW frame buffer size callback function. This is normally set using 
/// the GLFW `glfwSetFramebufferSizeCallback` function, but instead we explicitly
/// handle window resizing in our state updates on the application side. Run this function 
/// whenever the frame buffer is resized.
/// 
fn glfw_framebuffer_size_callback(context: &mut glh::GLContext, camera: &mut Camera, width: u32, height: u32) {
    context.width = width;
    context.height = height;

    let aspect = context.width as f32 / context.height as f32;
    camera.aspect = aspect;
    camera.proj_mat = math::perspective((camera.fov, aspect, camera.near, camera.far));
    unsafe {
        gl::Viewport(0, 0, context.width as i32, context.height as i32);
    }
}


#[allow(unused_variables)]
fn main() {
    let mut context = match glh::start_gl(720, 480, GL_LOG_FILE) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to Initialize OpenGL context. Got error:");
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    let text_font_atlas = load_text_font_atlas();
    let title_font_atlas = load_title_font_atlas();

    let (
        gp_sp,
        gp_view_mat_loc,
        gp_proj_mat_loc) = create_ground_plane_shaders(&context);
    
    let (
        ground_plane_points_vbo,
        ground_plane_points_vao) = create_ground_plane_geometry(&context);

    // Texture for the ground plane.
    let mut gp_tex = 0;
    load_texture(&asset_file(GROUND_PLANE_TEX), &mut gp_tex, gl::REPEAT);
    assert!(gp_tex > 0);

    /* --------------------------- TITLE SCREEN --------------------------- */
    let (
        title_screen_sp,
        title_screen_sp_color_loc) = create_title_screen_shaders(&context);

    // Screen text.
    let (
        string_vp_vbo,
        string_vt_vbo,
        string_vao,
        string_points
    ) = create_title_screen_geometry(
        &context, &text_font_atlas, "Press ENTER to continue", -0.65, -0.40, 40.0
    );

    // Font sheet for the title screen text.
    let mut text_screen_tex = 0;
    load_texture(&asset_file(TEXT_FONT_SHEET), &mut text_screen_tex, gl::CLAMP_TO_EDGE);
    assert!(text_screen_tex > 0);

    // Title text.
    let (
        title_vp_vbo,
        title_vt_vbo,
        title_vao,
        title_points
    ) = create_title_screen_geometry(
        &context, &title_font_atlas, "STALLMANIFOLD", -0.90, 0.4, 256.0
    );

    // Font sheet for the title text on the title screen.
    let mut title_screen_tex = 0;
    load_texture(&asset_file(TITLE_FONT_SHEET), &mut title_screen_tex, gl::CLAMP_TO_EDGE);
    assert!(title_screen_tex > 0);
    /* ------------------------- END TITLE SCREEN ------------------------- */

    let (
        cube_sp, 
        cube_view_mat_location,
        cube_proj_mat_location) = create_cube_map_shaders(&context);

    let cube_vao = create_cube_map_geometry();
    assert!(cube_vao > 0);

    // Texture for the cube map.
    let mut cube_map_texture = 0;
    create_cube_map(
        &asset_file(FRONT), &asset_file(BACK), &asset_file(TOP),
        &asset_file(BOTTOM), &asset_file(LEFT), &asset_file(RIGHT),
        &mut cube_map_texture
    );
    assert!(cube_map_texture > 0);

    let mut camera = create_camera(context.width, context.height);

    unsafe {
        gl::UseProgram(gp_sp);
        gl::UniformMatrix4fv(gp_view_mat_loc, 1, gl::FALSE, camera.view_mat.as_ptr());
        gl::UniformMatrix4fv(gp_proj_mat_loc, 1, gl::FALSE, camera.proj_mat.as_ptr());
    }

    unsafe {
        gl::UseProgram(cube_sp);
        gl::UniformMatrix4fv(cube_view_mat_location, 1, gl::FALSE, camera.rot_mat_inv.inverse().as_ptr());
        gl::UniformMatrix4fv(cube_proj_mat_location, 1, gl::FALSE, camera.proj_mat.as_ptr());
    }

    unsafe {
        // Enable depth-testing.
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
        gl::FrontFace(gl::CCW);
        gl::ClearColor(0.2, 0.2, 0.2, 1.0); // grey background to help spot mistakes
        gl::Viewport(0, 0, context.width as i32, context.height as i32);
    }

    /* -------------------------- RENDERING LOOP --------------------------- */
    while !context.window.should_close() {
        let elapsed_seconds = glh::update_timers(&mut context);
        glh::update_fps_counter(&mut context);

        let (width, height) = context.window.get_framebuffer_size();
        if (width != context.width as i32) && (height != context.height as i32) {
            glfw_framebuffer_size_callback(&mut context, &mut camera, width as u32, height as u32);
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Viewport(0, 0, context.width as i32, context.height as i32);

            // Draw the sky box using the cube map texture.
            gl::DepthMask(gl::FALSE);
            gl::UseProgram(cube_sp);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, cube_map_texture);
            gl::BindVertexArray(cube_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::DepthMask(gl::TRUE);

            // Draw the ground plane.
            gl::UseProgram(gp_sp);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, gp_tex);
            gl::BindVertexArray(ground_plane_points_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            
            // Draw the title screen. Disable depth testing and enable 
            // alpha blending to do so.
            gl::Disable(gl::DEPTH_TEST);
            gl::UseProgram(title_screen_sp);
            gl::ActiveTexture(gl::TEXTURE0);
            
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::BindTexture(gl::TEXTURE_2D, title_screen_tex);
            gl::BindVertexArray(title_vao);
            gl::Uniform4f(title_screen_sp_color_loc, TITLE_COLOR[0], TITLE_COLOR[1], TITLE_COLOR[2], 1.0);
            gl::DrawArrays(gl::TRIANGLES, 0, title_points as i32);
            gl::Disable(gl::BLEND);

            gl::BindTexture(gl::TEXTURE_2D, text_screen_tex);
            gl::BindVertexArray(string_vao);
            gl::Uniform4f(title_screen_sp_color_loc, TEXT_COLOR[0], TEXT_COLOR[2], TEXT_COLOR[2], 1.0);
            gl::DrawArrays(gl::TRIANGLES, 0, string_points as i32);
            gl::Enable(gl::DEPTH_TEST);
        }

        context.glfw.poll_events();

        /* ------------------------- UPDATE GAME STATE ------------------------ */
        // Camera control keys.
        let mut cam_moved = false;
        let mut move_to = math::vec3((0.0, 0.0, 0.0));
        let mut cam_yaw = 0.0;
        let mut cam_pitch = 0.0;
        let mut cam_roll = 0.0;
        match context.window.get_key(Key::A) {
            Action::Press | Action::Repeat => {
                move_to.x -= camera.cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::D) {
            Action::Press | Action::Repeat => {
                move_to.x += camera.cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::Q) {
            Action::Press | Action::Repeat => {
                move_to.y += camera.cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::E) {
            Action::Press | Action::Repeat => {
                move_to.y -= camera.cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::W) {
            Action::Press | Action::Repeat => {
                move_to.z -= camera.cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::S) {
            Action::Press | Action::Repeat => {
                move_to.z += camera.cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::Left) {
            Action::Press | Action::Repeat => {
                cam_yaw += camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_yaw = Quaternion::from_axis_deg(cam_yaw, math::vec3(camera.up));
                camera.axis = q_yaw * &camera.axis;
            }
            _ => {}
        }
        match context.window.get_key(Key::Right) {
            Action::Press | Action::Repeat => {
                cam_yaw -= camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_yaw = Quaternion::from_axis_deg(cam_yaw, math::vec3(camera.up));
                camera.axis = q_yaw * &camera.axis;
            }
            _ => {}
        }
        match context.window.get_key(Key::Up) {
            Action::Press | Action::Repeat => {
                cam_pitch += camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_pitch = Quaternion::from_axis_deg(cam_pitch, math::vec3(camera.rgt));
                camera.axis = q_pitch * &camera.axis;
            }
            _ => {}
        }
        match context.window.get_key(Key::Down) {
            Action::Press | Action::Repeat => {
                cam_pitch -= camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_pitch = Quaternion::from_axis_deg(cam_pitch, math::vec3(camera.rgt));
                camera.axis = q_pitch * &camera.axis;
            }
            _ => {}
        }
        match context.window.get_key(Key::Z) {
            Action::Press | Action::Repeat => {
                cam_roll -= camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_roll = Quaternion::from_axis_deg(cam_roll, math::vec3(camera.fwd));
                camera.axis = q_roll * &camera.axis;
            }
            _ => {}
        }
        match context.window.get_key(Key::C) {
            Action::Press | Action::Repeat => {
                cam_roll += camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_roll = Quaternion::from_axis_deg(cam_roll, math::vec3(camera.fwd));
                camera.axis = q_roll * &camera.axis;
            }
            _ => {}
        }
        match context.window.get_key(Key::Backspace) {
            Action::Press | Action::Repeat => {
                reset_camera_to_default(&context, &mut camera);
                cam_moved = true;
            }
            _ => {}
        }

        // Update view matrix.
        if cam_moved {
            // Recalculate local axes so we can move fwd in the direction the camera is pointing.
            camera.rot_mat_inv = Matrix4::from(camera.axis);
            camera.fwd = camera.rot_mat_inv * math::vec4((0.0, 0.0, -1.0, 0.0));
            camera.rgt = camera.rot_mat_inv * math::vec4((1.0, 0.0,  0.0, 0.0));
            camera.up  = camera.rot_mat_inv * math::vec4((0.0, 1.0,  0.0, 0.0));

            camera.cam_pos += math::vec3(camera.fwd) * -move_to.z;
            camera.cam_pos += math::vec3(camera.up)  *  move_to.y;
            camera.cam_pos += math::vec3(camera.rgt) *  move_to.x;
            camera.trans_mat_inv = Matrix4::from_translation(camera.cam_pos);

            camera.view_mat = camera.rot_mat_inv.inverse() * camera.trans_mat_inv.inverse();
            unsafe {
                gl::UseProgram(gp_sp);
                gl::UniformMatrix4fv(gp_view_mat_loc, 1, gl::FALSE, camera.view_mat.as_ptr());

                // Cube map view matrix has rotation, but not translation. It moves with the camera.
                gl::UseProgram(cube_sp);
                gl::UniformMatrix4fv(cube_view_mat_location, 1, gl::FALSE, camera.rot_mat_inv.inverse().as_ptr());
            }
        }

        // Check whether the user signaled GLFW to close the window.
        match context.window.get_key(Key::Escape) {
            Action::Press | Action::Repeat => {
                context.window.set_should_close(true);
            }
            _ => {}
        }
        /* ----------------------- END UPDATE GAME STATE ----------------------- */

        context.window.swap_buffers();
    }
    /* ---------------------- END RENDERING LOOP ----------------------------- */
}
