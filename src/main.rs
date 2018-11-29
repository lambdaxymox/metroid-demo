extern crate glfw;
extern crate chrono;
extern crate stb_image;
extern crate cgmath;
extern crate wavefront;
extern crate serde;
extern crate serde_json;
extern crate toml;
extern crate log;
extern crate file_logger;

#[macro_use]
extern crate serde_derive;

mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

mod config;
mod font_atlas;
mod gl_helpers;
mod camera;
mod obj;

use glfw::{Action, Context, Key};
use gl::types::{GLenum, GLfloat, GLint, GLsizeiptr, GLvoid, GLuint};

use stb_image::image;
use stb_image::image::LoadResult;

use std::env;
use std::mem;
use std::ptr;
use std::process;
use std::path::{Path, PathBuf};

use font_atlas::FontAtlas;

use gl_helpers as glh;
use cgmath as math;
use math::{Matrix4, Quaternion, AsArray};
use camera::Camera;
use log::{info};


// OpenGL extension constants.
const GL_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;
const GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;

#[cfg(not(feature = "build_for_install"))]
const CONFIG_HOME: &str = "config";

#[cfg(feature = "build_for_install")]
const CONFIG_HOME: &str = ".config";

#[cfg(feature = "build_for_install")]
const DATA_DIR: &str = ".local/share/metroid-demo";

#[cfg(feature = "build_for_install")]
const BIN_DIR: &str = ".local/bin";

const CONFIG_FILE: &str = "metroid-demo.toml";

// Text colors.
const TITLE_COLOR: [f32; 3] = [1.0, 1.0, 1.0];
const TEXT_COLOR: [f32; 3] = [139 as f32 / 255 as f32, 193 as f32 / 255 as f32, 248 as f32 / 255 as f32];


fn load_text_font_atlas(context: &Game) -> FontAtlas {
    font_atlas::load(&context.asset_file("text_font2048x2048.json")).unwrap()
}

fn load_title_font_atlas(context: &Game) -> FontAtlas {
    font_atlas::load(&context.asset_file("title_font2048x2048.json")).unwrap()
}

///
/// Create the shaders for rendering text.
///
fn create_title_screen_shaders(context: &Game) -> (GLuint, GLint) {
    let title_screen_sp = glh::create_program_from_files(
        &context.gl,
        &context.shader_file("title_screen.vert.glsl"),
        &context.shader_file("title_screen.frag.glsl")
    ).unwrap();
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
    context: &Game, shader: GLuint,
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
        &context.gl, text, &font_atlas,
        x_pos, y_pos, pixel_scale, 
        &mut string_vp_vbo, &mut string_vt_vbo, &mut string_point_count
    );

    let string_vp_loc = unsafe { gl::GetAttribLocation(shader, glh::gl_str("vp").as_ptr()) };
    assert!(string_vp_loc > -1);
    let string_vp_loc = string_vp_loc as u32;

    let string_vt_loc = unsafe { gl::GetAttribLocation(shader, glh::gl_str("vt").as_ptr()) };
    assert!(string_vt_loc > -1);
    let string_vt_loc = string_vt_loc as u32;

    let mut string_vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut string_vao);
        gl::BindVertexArray(string_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, string_vp_vbo);
        gl::VertexAttribPointer(string_vp_loc, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(string_vp_loc);
        gl::BindBuffer(gl::ARRAY_BUFFER, string_vt_vbo);
        gl::VertexAttribPointer(string_vt_loc, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(string_vt_loc);
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
        let address = atlas.glyph_coords[&ch_i];
        
        let s = (address.column as f32) * (1.0 / (atlas.columns as f32));
        let t = ((address.row + 1) as f32) * (1.0 / (atlas.rows as f32));

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
fn create_cube_map_geometry(context: &Game, shader: GLuint) -> GLuint {
    let cube_map = obj::load_file(&context.asset_file("cube_map.obj")).unwrap();

    let mut cube_map_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut cube_map_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, cube_map_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (3 * mem::size_of::<GLfloat>() * cube_map.len()) as GLsizeiptr,
            cube_map.points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
    }

    let cube_map_vp_loc = unsafe { gl::GetAttribLocation(shader, glh::gl_str("vp").as_ptr()) };
    assert!(cube_map_vp_loc > -1);
    let cube_map_vp_loc = cube_map_vp_loc as u32;

    let mut cube_map_vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut cube_map_vao);
        gl::BindVertexArray(cube_map_vao);
        gl::EnableVertexAttribArray(cube_map_vp_loc);
        gl::BindBuffer(gl::ARRAY_BUFFER, cube_map_vbo);
        gl::VertexAttribPointer(cube_map_vp_loc, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
    }

    cube_map_vao
}

///
/// Load one of the cube map sides into a cube map texture.
///
fn load_cube_map_side(texture: GLuint, side_target: GLenum, file_name: &Path) -> bool {
    unsafe {
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture);
    }

    let force_channels = 4;
    let image_data = match image::load_with_depth(&file_name, force_channels, false) {
        LoadResult::ImageU8(image_data) => image_data,
        LoadResult::Error(_) => {
            let disp = file_name.display();
            eprintln!("ERROR: could not load {}", disp);
            return false;
        }
        LoadResult::ImageF32(_) => {
            let disp = file_name.display();
            eprintln!("ERROR: Tried to load an image as byte vectors, got f32: {}", disp);
            return false;
        }
    };

    let width = image_data.width;
    let height = image_data.height;

    // Check that the image size is a power of two.
    if (width & (width - 1)) != 0 || (height & (height - 1)) != 0 {
        let disp = file_name.display();
        eprintln!("WARNING: Texture {} lacks dimensions that are a power of two", disp);
    }

    // Copy image data into the target side of the cube map.
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
    front: &Path, back: &Path, top: &Path,
    bottom: &Path, left: &Path, right: &Path, tex_cube: &mut GLuint) {
    
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
fn create_cube_map_shaders(context: &Game) -> (GLuint, GLint, GLint) {
    let cube_sp = glh::create_program_from_files(
        &context.gl,
        &context.shader_file("cube.vert.glsl"),
        &context.shader_file("cube.frag.glsl")
    ).unwrap();
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
fn create_ground_plane_shaders(context: &Game) -> (GLuint, GLint, GLint) {
    // Here I used negative y from the buffer as the z value so that it was on
    // the floor but also that the 'front' was on the top side. Also note how I
    // work out the texture coordinates, st, from the vertex point position.
    let gp_sp = glh::create_program_from_files(
        &context.gl,
        &context.shader_file("ground_plane.vert.glsl"),
        &context.shader_file("ground_plane.frag.glsl")
    ).unwrap();
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
fn create_ground_plane_geometry(context: &Game, shader: GLuint) -> (GLuint, GLuint) {
    let mesh = obj::load_file(&context.asset_file("ground_plane.obj")).unwrap();

    let mut gp_vp_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut gp_vp_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, gp_vp_vbo);
        gl::BufferData( 
            gl::ARRAY_BUFFER, (3 * mem::size_of::<GLfloat>() * mesh.len()) as GLsizeiptr,
            mesh.points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
    }
    assert!(gp_vp_vbo > 0);

    let gp_vp_loc = unsafe { gl::GetAttribLocation(shader, glh::gl_str("vp").as_ptr()) };
    assert!(gp_vp_loc > -1);
    let gp_vp_loc = gp_vp_loc as u32;

    let mut gp_vp_vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut gp_vp_vao);
        gl::BindVertexArray(gp_vp_vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, gp_vp_vbo);
        gl::VertexAttribPointer(gp_vp_loc, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(gp_vp_loc);
    }
    assert!(gp_vp_vao > 0);

    (gp_vp_vbo, gp_vp_vao)
}

///
/// Initialize the camera to default position and orientation.
///
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
/// Load textures into graphics memory.
///
fn load_texture<P: AsRef<Path>>(
    file_name: P, tex: &mut GLuint, wrapping_mode: GLuint) -> bool {

    let force_channels = 4;
    let mut image_data = match image::load_with_depth(&file_name, force_channels, false) {
        LoadResult::ImageU8(image_data) => image_data,
        LoadResult::Error(_) => {
            let disp = file_name.as_ref().display();
            eprintln!("ERROR: could not load {}", disp);
            return false;
        }
        LoadResult::ImageF32(_) => {
            let disp = file_name.as_ref().display();
            eprintln!("ERROR: Tried to load an image as byte vectors, got f32: {}", disp);
            return false;
        }
    };

    let width = image_data.width;
    let height = image_data.height;

    // Check that the image size is a power of two.
    if (width & (width - 1)) != 0 || (height & (height - 1)) != 0 {
        let disp = file_name.as_ref().display();
        eprintln!("WARNING: texture {} is not power-of-2 dimensions", disp);
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

struct Game {
    config: config::ProgramConfig,
    gl: glh::GLContext,
}

impl Game {
    fn new(config: config::ProgramConfig, gl_context: glh::GLContext) -> Game {
        Game { config: config, gl: gl_context }
    }

    fn shader_file<P: AsRef<Path>>(&self, file: P) -> PathBuf {
        let shader_path = Path::new(&self.config.shader_path);
        let shader_version = Path::new(&self.config.shader_version);
        let file_path = shader_path.join(shader_version).join(file);

        file_path
    }

    fn asset_file<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        Path::new(&self.config.asset_path).join(path)
    }
}

#[cfg(feature = "build_for_install")]
#[inline]
fn __path_config() -> config::PathConfig {
    let st = env::var("HOME").unwrap();
    let home = Path::new(&st);
    let config_home = Path::new(CONFIG_HOME);
    let bin_dir = Path::new(BIN_DIR);
    let data_dir = Path::new(DATA_DIR);

    config::PathConfig::new(
        home.join(config_home), home.join(bin_dir), home.join(data_dir)
    )
}

#[cfg(not(feature = "build_for_install"))]
#[inline]
fn __path_config() -> config::PathConfig {
    config::PathConfig::new(
        PathBuf::from(CONFIG_HOME), PathBuf::from("."), PathBuf::from(".")
    )
}

fn load_config() -> config::ProgramConfig {
    let path_config = __path_config();
    let file_config = config::load(path_config.config_home.join(CONFIG_FILE)).unwrap();

    config::ProgramConfig::new(path_config, file_config)
}

///
/// Initialize the logger.
///
fn init_logger(log_file: &str) {
    file_logger::init(log_file).expect("Failed to initialize logger.");
    info!("OpenGL application log.");
    info!("build version: ??? ?? ???? ??:??:??\n\n");
}

fn start() -> Game {
    let config = load_config();
    init_logger(config.log_file.to_str().unwrap());
    let gl_context = match glh::start_gl(720, 480) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to Initialize OpenGL context. Got error:");
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    Game::new(config, gl_context)
}

#[allow(unused_variables)]
fn main() {
    let mut context = start();

    let text_font_atlas = load_text_font_atlas(&context);
    let title_font_atlas = load_title_font_atlas(&context);

    let (
        gp_sp,
        gp_view_mat_loc,
        gp_proj_mat_loc) = create_ground_plane_shaders(&context);
    
    let (
        ground_plane_points_vbo,
        ground_plane_points_vao) = create_ground_plane_geometry(&context, gp_sp);

    // Texture for the ground plane.
    let mut gp_tex = 0;
    load_texture(&context.asset_file("tile_rock_planet256x256.png"), &mut gp_tex, gl::REPEAT);
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
        &context, title_screen_sp, &text_font_atlas, "Press ENTER to continue", -0.65, -0.40, 40.0
    );

    // Font sheet for the title screen text.
    let mut text_screen_tex = 0;
    load_texture(&context.asset_file("text_font2048x2048.png"), &mut text_screen_tex, gl::CLAMP_TO_EDGE);
    assert!(text_screen_tex > 0);

    // Title text.
    let (
        title_vp_vbo,
        title_vt_vbo,
        title_vao,
        title_points
    ) = create_title_screen_geometry(
        &context, title_screen_sp, &title_font_atlas, "STALLMANIFOLD", -0.90, 0.4, 256.0
    );

    // Font sheet for the title text on the title screen.
    let mut title_screen_tex = 0;
    load_texture(&context.asset_file("title_font2048x2048.png"), &mut title_screen_tex, gl::CLAMP_TO_EDGE);
    assert!(title_screen_tex > 0);
    /* ------------------------- END TITLE SCREEN ------------------------- */

    let (
        cube_sp, 
        cube_view_mat_location,
        cube_proj_mat_location) = create_cube_map_shaders(&context);

    let cube_vao = create_cube_map_geometry(&context, cube_sp);
    assert!(cube_vao > 0);

    // Texture for the cube map.
    let mut cube_map_texture = 0;
    let config = &context.config;
    create_cube_map(
        &context.asset_file("skybox_panel.png"), &context.asset_file("skybox_panel.png"),
        &context.asset_file("skybox_panel.png"), &context.asset_file("skybox_panel.png"),
        &context.asset_file("skybox_panel.png"), &context.asset_file("skybox_panel.png"),
        &mut cube_map_texture
    );
    assert!(cube_map_texture > 0);

    let mut camera = create_camera(context.gl.width, context.gl.height);

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
        gl::Viewport(0, 0, context.gl.width as i32, context.gl.height as i32);
    }

    /* -------------------------- RENDERING LOOP --------------------------- */
    while !context.gl.window.should_close() {
        let elapsed_seconds = glh::update_timers(&mut context.gl);
        glh::update_fps_counter(&mut context.gl);

        let (width, height) = context.gl.window.get_framebuffer_size();
        if (width != context.gl.width as i32) && (height != context.gl.height as i32) {
            glfw_framebuffer_size_callback(&mut context.gl, &mut camera, width as u32, height as u32);
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Viewport(0, 0, context.gl.width as i32, context.gl.height as i32);

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

        context.gl.glfw.poll_events();

        /* ------------------------- UPDATE GAME STATE ------------------------ */
        // Camera control keys.
        let mut cam_moved = false;
        let mut move_to = math::vec3((0.0, 0.0, 0.0));
        let mut cam_yaw = 0.0;
        let mut cam_pitch = 0.0;
        let mut cam_roll = 0.0;
        match context.gl.window.get_key(Key::A) {
            Action::Press | Action::Repeat => {
                move_to.x -= camera.cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.gl.window.get_key(Key::D) {
            Action::Press | Action::Repeat => {
                move_to.x += camera.cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.gl.window.get_key(Key::Q) {
            Action::Press | Action::Repeat => {
                move_to.y += camera.cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.gl.window.get_key(Key::E) {
            Action::Press | Action::Repeat => {
                move_to.y -= camera.cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.gl.window.get_key(Key::W) {
            Action::Press | Action::Repeat => {
                move_to.z -= camera.cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.gl.window.get_key(Key::S) {
            Action::Press | Action::Repeat => {
                move_to.z += camera.cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.gl.window.get_key(Key::Left) {
            Action::Press | Action::Repeat => {
                cam_yaw += camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_yaw = Quaternion::from_axis_deg(cam_yaw, math::vec3(camera.up));
                camera.axis = q_yaw * &camera.axis;
            }
            _ => {}
        }
        match context.gl.window.get_key(Key::Right) {
            Action::Press | Action::Repeat => {
                cam_yaw -= camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_yaw = Quaternion::from_axis_deg(cam_yaw, math::vec3(camera.up));
                camera.axis = q_yaw * &camera.axis;
            }
            _ => {}
        }
        match context.gl.window.get_key(Key::Up) {
            Action::Press | Action::Repeat => {
                cam_pitch += camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_pitch = Quaternion::from_axis_deg(cam_pitch, math::vec3(camera.rgt));
                camera.axis = q_pitch * &camera.axis;
            }
            _ => {}
        }
        match context.gl.window.get_key(Key::Down) {
            Action::Press | Action::Repeat => {
                cam_pitch -= camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_pitch = Quaternion::from_axis_deg(cam_pitch, math::vec3(camera.rgt));
                camera.axis = q_pitch * &camera.axis;
            }
            _ => {}
        }
        match context.gl.window.get_key(Key::Z) {
            Action::Press | Action::Repeat => {
                cam_roll -= camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_roll = Quaternion::from_axis_deg(cam_roll, math::vec3(camera.fwd));
                camera.axis = q_roll * &camera.axis;
            }
            _ => {}
        }
        match context.gl.window.get_key(Key::C) {
            Action::Press | Action::Repeat => {
                cam_roll += camera.cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_roll = Quaternion::from_axis_deg(cam_roll, math::vec3(camera.fwd));
                camera.axis = q_roll * &camera.axis;
            }
            _ => {}
        }
        match context.gl.window.get_key(Key::Backspace) {
            Action::Press | Action::Repeat => {
                reset_camera_to_default(&context.gl, &mut camera);
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
        match context.gl.window.get_key(Key::Escape) {
            Action::Press | Action::Repeat => {
                context.gl.window.set_should_close(true);
            }
            _ => {}
        }
        /* ----------------------- END UPDATE GAME STATE ----------------------- */

        context.gl.window.swap_buffers();
    }
    /* ---------------------- END RENDERING LOOP ----------------------------- */
}
