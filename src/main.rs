extern crate gl;
extern crate glfw;
extern crate chrono;
extern crate stb_image;

#[macro_use]
mod logger;

mod gl_helpers;
mod math;
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
use math::{Matrix4, Versor};
use camera::Camera;

const GL_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;
const GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;

const GL_LOG_FILE: &str = "gl.log";
const FRONT: &str = "assets/skybox-panel.png";
const BACK: &str = "assets/skybox-panel.png";
const LEFT: &str = "assets/skybox-panel.png";
const RIGHT: &str = "assets/skybox-panel.png";
const TOP: &str = "assets/skybox-panel.png";
const BOTTOM: &str = "assets/skybox-panel.png";
const FONT_SHEET: &str = "assets/font1684x1684.png";
const GROUND_PLANE_TEX: &str = "assets/tile-rock-planet256x256.png";

const TEXT_COLOR: [f32; 3] = [194 as f32 / 255 as f32, 210 as f32 / 255 as f32, 234 as f32 / 255 as f32];


struct FontAtlas {
    glyph_y_offsets: HashMap<char, f32>,
    glyph_widths: HashMap<char, f32>,
    coords: HashMap<char, (usize, usize)>,
    bitmap: Vec<u8>,
    rows: usize,
    columns: usize,
}

fn load_font_atlas() -> FontAtlas {
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
    let rows = 7;
    let columns = 7;

    FontAtlas {
        glyph_y_offsets: glyph_y_offsets,
        glyph_widths: glyph_widths,
        coords: coords,
        bitmap: bitmap,
        rows: rows,
        columns: columns,
    }
}

fn create_title_screen_shaders(context: &glh::GLContext) -> (GLuint, GLint) {
    let sp = glh::create_program_from_files(
        context, "shaders/title_screen.vert.glsl", "shaders/title_screen.frag.glsl"
    );
    assert!(sp > 0);

    let sp_text_color_loc = unsafe { 
        gl::GetUniformLocation(sp, "text_color".as_ptr() as *const i8)
    };
    assert!(sp_text_color_loc > -1);

    (sp, sp_text_color_loc)
}

fn create_title_screen_geometry(
    context: &glh::GLContext, font_atlas: &FontAtlas, text: &str) -> (GLuint, GLuint, GLuint, usize) {
    
    let mut string_vp_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut string_vp_vbo);
    }

    let mut string_vt_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut string_vt_vbo);
    }

    let x_pos: GLfloat = -0.75;
    let y_pos: GLfloat = -0.4;
    let pixel_scale: GLfloat = 42.0;
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

fn create_cube_map_shaders(context: &glh::GLContext) -> (GLuint, GLint, GLint) {
    let cube_sp = glh::create_program_from_files(
        &context, "shaders/cube.vert.glsl", "shaders/cube.frag.glsl"
    );
    assert!(cube_sp > 0);
    // NOTE: This view matrix should *NOT* contain camera translation.
    let cube_view_mat_location = unsafe {
        gl::GetUniformLocation(cube_sp, "view".as_ptr() as *const i8)
    };
    assert!(cube_view_mat_location > -1);

    let cube_proj_mat_location = unsafe {
        gl::GetUniformLocation(cube_sp, "proj".as_ptr() as *const i8)
    };
    assert!(cube_proj_mat_location > -1);

    (cube_sp, cube_view_mat_location, cube_proj_mat_location)
}

fn create_ground_plane_shaders(context: &glh::GLContext) -> (GLuint,  GLint, GLint) {
    // Here I used negative y from the buffer as the z value so that it was on
    // the floor but also that the 'front' was on the top side. also note how I
    // work out the texture coordinates, st, from the vertex point position.
    let gp_sp = glh::create_program_from_files(context, "shaders/gp.vert.glsl", "shaders/gp.frag.glsl");

    // Get uniform locations of camera view and projection matrices.
    let gp_view_mat_loc = unsafe { gl::GetUniformLocation(gp_sp, "view".as_ptr() as *const i8) };
    assert!(gp_view_mat_loc > -1);

    let gp_proj_mat_loc = unsafe { gl::GetUniformLocation(gp_sp, "proj".as_ptr() as *const i8) };
    assert!(gp_proj_mat_loc > -1);

    (gp_sp, gp_view_mat_loc, gp_proj_mat_loc)
}

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

fn create_camera(context: &glh::GLContext) -> Camera {
    let near = 0.1;
    let far = 100.0;
    let fov = 67.0;
    let aspect = context.width as f32 / context.height as f32;

    let cam_speed: GLfloat = 3.0;
    let cam_yaw_speed: GLfloat = 50.0;

    let fwd = math::vec4((0.0, 0.0, -1.0, 0.0));
    let rgt = math::vec4((1.0, 0.0,  0.0, 0.0));
    let up  = math::vec4((0.0, 1.0,  0.0, 0.0));
    let cam_pos = math::vec3((0.0, 0.0, 5.0));
    
    let axis = Versor::from_axis_deg(0.0, math::vec3((1.0, 0.0, 0.0)));

    Camera::new(near, far, fov, aspect, cam_speed, cam_yaw_speed, cam_pos, fwd, rgt, up, axis)
}

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
    // TODO: Check this against my OpenGL extension dependencies.
    unsafe {
        gl::GetFloatv(GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut max_aniso);
        // Set the maximum!
        gl::TexParameterf(gl::TEXTURE_2D, GL_TEXTURE_MAX_ANISOTROPY_EXT, max_aniso);
    }

    true
}

#[allow(unused_variables)]
fn main() {
    let mut context = match glh::start_gl(GL_LOG_FILE) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to Initialize OpenGL context. Got error:");
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    let font_atlas = load_font_atlas();

    let (
        gp_sp,
        gp_view_mat_loc,
        gp_proj_mat_loc) = create_ground_plane_shaders(&context);

    let (
        ground_plane_points_vbo,
        ground_plane_points_vao) = create_ground_plane_geometry(&context);

    // Texture for the ground plane.
    let mut gp_tex = 0;
    load_texture(GROUND_PLANE_TEX, &mut gp_tex, gl::REPEAT);
    assert!(gp_tex > 0);

    let (
        title_screen_sp,
        title_screen_sp_color_loc) = create_title_screen_shaders(&context);

    let (
        string_vp_vbo,
        string_vt_vbo,
        string_vao,
        string_points) = create_title_screen_geometry(&context, &font_atlas, "Press ENTER to continue");

    // Font sheet for the title screen text.
    let mut title_screen_tex = 0;
    load_texture(FONT_SHEET, &mut title_screen_tex, gl::CLAMP_TO_EDGE);
    assert!(title_screen_tex > 0);

    let (
        cube_sp, 
        cube_view_mat_location,
        cube_proj_mat_location) = create_cube_map_shaders(&context);

    let cube_vao = create_cube_map_geometry();
    assert!(cube_vao > 0);

    // Texture for the cube map.
    let mut cube_map_texture = 0;
    create_cube_map(FRONT, BACK, TOP, BOTTOM, LEFT, RIGHT, &mut cube_map_texture);
    assert!(cube_map_texture > 0);

    let mut camera = create_camera(&context);

    unsafe {
        gl::UseProgram(gp_sp);
        gl::UniformMatrix4fv(gp_view_mat_loc, 1, gl::FALSE, camera.view_mat.as_ptr());
        gl::UniformMatrix4fv(gp_proj_mat_loc, 1, gl::FALSE, camera.proj_mat.as_ptr());
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

    /* ********************** RENDERING LOOP ****************************** */
    while !context.window.should_close() {
        let elapsed_seconds = glh::update_timers(&mut context);
        glh::update_fps_counter(&mut context);

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
            gl::Enable(gl::BLEND);
            gl::UseProgram(title_screen_sp);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, title_screen_tex);
            gl::BindVertexArray(string_vao);
            gl::Uniform4f(title_screen_sp_color_loc, TEXT_COLOR[0], TEXT_COLOR[2], TEXT_COLOR[2], 1.0);
            gl::DrawArrays(gl::TRIANGLES, 0, string_points as i32);
            gl::Enable(gl::DEPTH_TEST);
            gl::Disable(gl::BLEND);
        }

        context.glfw.poll_events();

        /* ********************** UPDATE GAME STATE ************************* */
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
                let q_yaw = Versor::from_axis_deg(cam_yaw, math::vec3(camera.up));
                camera.axis = q_yaw * &camera.axis;
            }
            _ => {}
        }
        match context.window.get_key(Key::Right) {
            Action::Press | Action::Repeat => {
                cam_yaw -= camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_yaw = Versor::from_axis_deg(cam_yaw, math::vec3(camera.up));
                camera.axis = q_yaw * &camera.axis;
            }
            _ => {}
        }
        match context.window.get_key(Key::Up) {
            Action::Press | Action::Repeat => {
                cam_pitch += camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_pitch = Versor::from_axis_deg(cam_pitch, math::vec3(camera.rgt));
                camera.axis = q_pitch * &camera.axis;
            }
            _ => {}
        }
        match context.window.get_key(Key::Down) {
            Action::Press | Action::Repeat => {
                cam_pitch -= camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_pitch = Versor::from_axis_deg(cam_pitch, math::vec3(camera.rgt));
                camera.axis = q_pitch * &camera.axis;
            }
            _ => {}
        }
        match context.window.get_key(Key::Z) {
            Action::Press | Action::Repeat => {
                cam_roll -= camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_roll = Versor::from_axis_deg(cam_roll, math::vec3(camera.fwd));
                camera.axis = q_roll * &camera.axis;
            }
            _ => {}
        }
        match context.window.get_key(Key::C) {
            Action::Press | Action::Repeat => {
                cam_roll += camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_roll = Versor::from_axis_deg(cam_roll, math::vec3(camera.fwd));
                camera.axis = q_roll * &camera.axis;
            }
            _ => {}
        }

        // update view matrix
        if cam_moved {
            // re-calculate local axes so can move fwd in dir cam is pointing
            camera.rot_mat_inv = camera.axis.to_mat4();
            camera.fwd = camera.rot_mat_inv * math::vec4((0.0, 0.0, -1.0, 0.0));
            camera.rgt = camera.rot_mat_inv * math::vec4((1.0, 0.0,  0.0, 0.0));
            camera.up  = camera.rot_mat_inv * math::vec4((0.0, 1.0,  0.0, 0.0));

            camera.cam_pos += math::vec3(camera.fwd) * -move_to.z;
            camera.cam_pos += math::vec3(camera.up)  *  move_to.y;
            camera.cam_pos += math::vec3(camera.rgt) *  move_to.x;
            camera.trans_mat_inv = Matrix4::identity().translate(&camera.cam_pos);

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
        /* ********************* END UPDATE GAME STATE ************************* */

        context.window.swap_buffers();
    }
    /* ******************* END RENDERING LOOP ****************************** */
}
