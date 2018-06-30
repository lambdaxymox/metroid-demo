extern crate gl;
extern crate glfw;
extern crate chrono;
extern crate stb_image;

#[macro_use]
mod logger;

mod gl_helpers;
mod math;

use glfw::{Action, Context, Key};
use gl::types::{GLchar, GLenum, GLfloat, GLint, GLsizeiptr, GLvoid, GLuint};

use stb_image::image;
use stb_image::image::LoadResult;

use std::collections::HashMap;
use std::mem;
use std::ptr;
use std::process;

use gl_helpers as glh;
use math::{Vec3, Mat4, Versor};

const GL_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;
const GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;

const GL_LOG_FILE: &str = "gl.log";
const FRONT: &str = "assets/skybox-panel.png";
const BACK: &str = "assets/skybox-panel.png";
const LEFT: &str = "assets/skybox-panel.png";
const RIGHT: &str = "assets/skybox-panel.png";
const TOP: &str = "assets/skybox-panel.png";
const BOTTOM: &str = "assets/skybox-panel.png";
const FONT_SHEET: &str = "asserts/font1684x1684.png";


struct FontAtlas {
    glyph_y_offsets: HashMap<char, f32>,
    glyph_widths: HashMap<char, f32>,
    coords: HashMap<char, (usize, usize)>,
    bitmap: Vec<u8>,
    rows: usize,
    columns: usize,
}

fn load_font_atlas() -> FontAtlas {
    unimplemented!()
}

fn create_title_screen_shaders(context: &glh::GLContext) -> (GLuint, GLint) {
    let sp = glh::create_program_from_files(
        context, "shaders/title_screen.vert.glsl", "title_screen.frag.glsl"
    );

    let sp_text_color_loc = unsafe { 
        gl::GetUniformLocation(sp, "text_color".as_ptr() as *const i8)
    };
    assert!(sp_text_color_loc > -1);

    (sp, sp_text_color_loc)
}

fn text_to_vbo(
    context: &glh::GLContext, st: &str, atlas: &FontAtlas,
    start_x: f32, start_y: f32, scale_px: f32,
    points_vbo: &mut GLuint, texcoords_vbo: &mut GLuint, point_count: &mut usize) {

    let mut points_temp = vec![0.0; 6 * 3 * mem::size_of::<GLfloat>() * st.len()];
    let mut texcoords_temp = vec![0.0; 6 * 2 * mem::size_of::<GLfloat>() * st.len()];

    // TODO:
    // Loop through string and generate texture coordinates from the
    // gylph offset table and glyph width table. The glyph offset table
    // calculates where the top left corner of a glyph is.
    // The glyph width table calculates how wide each glyph is.
    for (i, ch_i) in st.chars().enumerate() {
        let (atlas_row, atlas_col) = atlas.coords[&ch_i];
        
        let s = (atlas_col as f32) * (1.0 / (atlas.columns as f32));
        let t = ((atlas_row + 1) as f32) * (1.0 / (atlas.rows as f32));

        let x_pos = start_x;
        let y_pos = start_y - scale_px / (context.height as f32) * atlas.glyph_y_offsets[&ch_i];

        points_temp[12 * i]     = x_pos;
        points_temp[12 * i + 1] = y_pos;
        points_temp[12 * i + 2] = x_pos;
        points_temp[12 * i + 3] = y_pos - scale_px / (context.height as f32);
        points_temp[12 * i + 4] = x_pos + scale_px / (context.width as f32);
        points_temp[12 * i + 5] = y_pos - scale_px / (context.height as f32);

        points_temp[12 * i + 6]  = x_pos + scale_px / (context.width as f32);
        points_temp[12 * i + 7]  = y_pos - scale_px / (context.height as f32);
        points_temp[12 * i + 8]  = x_pos + scale_px / (context.width as f32);
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
fn make_cube_map_mesh() -> GLuint {
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

fn create_ground_plane_shaders(view_mat: &Mat4, proj_mat: &Mat4) -> (GLuint,  GLint, GLint) {
    // Here I used negative y from the buffer as the z value so that it was on
    // the floor but also that the 'front' was on the top side. also note how I
    // work out the texture coordinates, st, from the vertex point position.
    let mut gp_vs_str = vec![0; 1024];
    if glh::parse_shader("shaders/gp.vert.glsl", &mut gp_vs_str).is_err() {
        panic!("Failed to parse ground plane vertex shader file.");
    }

    let mut gp_fs_str = vec![0; 1024];
    if glh::parse_shader("shaders/gp.frag.glsl", &mut gp_fs_str).is_err() {
        panic!("Failed to parse ground plane fragment shader file.");
    }
    
    unsafe {
        let gp_vs = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(gp_vs, 1, &(gp_vs_str.as_ptr() as *const GLchar), ptr::null());
        gl::CompileShader(gp_vs);
        assert!(gp_vs > 0);

        let gp_fs = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(gp_fs, 1, &(gp_fs_str.as_ptr() as *const GLchar), ptr::null());
        gl::CompileShader(gp_fs);
        assert!(gp_fs > 0);

        let gp_sp = gl::CreateProgram();
        gl::AttachShader(gp_sp, gp_vs);
        gl::AttachShader(gp_sp, gp_fs);
        gl::LinkProgram(gp_sp);
        assert!(gp_sp > 0);

        // Get uniform locations of camera view and projection matrices.
        let gp_view_mat_loc = gl::GetUniformLocation(gp_sp, "view".as_ptr() as *const i8);
        assert!(gp_view_mat_loc > -1);

        let gp_proj_mat_loc = gl::GetUniformLocation(gp_sp, "proj".as_ptr() as *const i8);
        assert!(gp_proj_mat_loc > -1);

        // Set defaults for matrices
        gl::UseProgram(gp_sp);
        gl::UniformMatrix4fv(gp_view_mat_loc, 1, gl::FALSE, view_mat.as_ptr());
        gl::UniformMatrix4fv(gp_proj_mat_loc, 1, gl::FALSE, proj_mat.as_ptr());

        (gp_sp, gp_view_mat_loc, gp_proj_mat_loc)
    }
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
    // TODO: Check this against my dependencies.
    unsafe {
        gl::GetFloatv(GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut max_aniso);
        // Set the maximum!
        gl::TexParameterf(gl::TEXTURE_2D, GL_TEXTURE_MAX_ANISOTROPY_EXT, max_aniso);
    }

    return true;
}

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

    /* ******************* GROUND PLANE GEOMETRY *************************/
    let ground_plane_points: [GLfloat; 18] = [
         20.0,  10.0, 0.0, -20.0,  10.0, 0.0, -20.0, -10.0, 0.0, 
        -20.0, -10.0, 0.0,  20.0, -10.0, 0.0,  20.0,  10.0, 0.0
    ];

    let mut ground_plane_points_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut ground_plane_points_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, ground_plane_points_vbo);
        gl::BufferData( 
            gl::ARRAY_BUFFER, (mem::size_of::<GLfloat>() * ground_plane_points.len()) as GLsizeiptr,
            ground_plane_points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
    }
    assert!(ground_plane_points_vbo > 0);

    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, ground_plane_points_vbo);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(0);
    }
    assert!(vao > 0);

    /* ******************* END GROUND PLANE GEOMETRY ******************** */
    /* ******************* TEXT BOX GEOMETRY **************************** */
    let mut string_vp_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut string_vp_vbo);
    }

    let mut string_vt_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut string_vt_vbo);
    }

    let x_pos: GLfloat = -0.75;
    let y_pos: GLfloat = 0.2;
    let pixel_scale: GLfloat = 64.0;
    let st = "Press ENTER to continue";
    let mut string_points = 0;
    text_to_vbo(
        &context, &st, &font_atlas, 
        x_pos, y_pos, pixel_scale, 
        &mut string_vp_vbo, &mut string_vt_vbo, &mut string_points
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

    let (title_screen_sp, title_screen_sp_colour_loc) = create_title_screen_shaders(&context);

    let mut title_screen_tex = 0;
    load_texture("assets/font1684x1684.png", &mut title_screen_tex, gl::CLAMP_TO_EDGE);
    assert!(title_screen_tex > 0);

    /* ******************* END TEXT BOX GEOMETRY ************************ */
    /* ****************************CUBE MAP ***************************** */
    let cube_vao = make_cube_map_mesh();
    assert!(cube_vao > 0);

    let mut cube_map_texture = 0;
    create_cube_map(FRONT, BACK, TOP, BOTTOM, LEFT, RIGHT, &mut cube_map_texture);
    assert!(cube_map_texture > 0);
    /* ************************* END CUBE MAP *************************** */
    // Cube map shaders
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


    let shader_program = glh::create_program_from_files(
        &context, "shaders/metroid_demo.vert.glsl", "shaders/metroid_demo.frag.glsl"
    );

    /*************************** CAMERA MODEL *****************************/
    let near = 0.1;
    let far = 100.0;
    let fov = 67.0;
    let aspect = context.width as f32 / context.height as f32;
    let proj_mat = Mat4::perspective(fov, aspect, near, far);

    // View matrix components.
    let cam_speed: GLfloat = 3.0;
    let cam_yaw_speed: GLfloat = 50.0;
    let mut cam_yaw: GLfloat = 0.0;
    let mut fwd = math::vec4((0.0, 0.0, -1.0, 0.0));
    let mut rgt = math::vec4((1.0, 0.0,  0.0, 0.0));
    let mut up  = math::vec4((0.0, 1.0,  0.0, 0.0));
    let mut cam_pos = math::vec3((0.0, 0.0, 5.0));
    let mut trans_mat_inv = Mat4::identity().translate(&cam_pos);
    
    let mut q = Versor::from_axis_deg(0.0, 1.0, 0.0, 0.0);
    let mut rot_mat_inv = q.to_mat4();

    let mut view_mat = rot_mat_inv.inverse() * trans_mat_inv.inverse();
    /************************ END CAMERA MODEL *****************************/


    // Load the shader program for the ground plane.
    let (gp_sp, gp_view_mat_loc, gp_proj_mat_loc) = create_ground_plane_shaders(&view_mat, &proj_mat);

    // Texture for the ground plane.
    let mut gp_tex = 0;
    load_texture("assets/tile-rock-planet256x256.png", &mut gp_tex, gl::REPEAT);
    assert!(gp_tex > 0);
    
    unsafe {
        gl::UseProgram(gp_sp);
        gl::UniformMatrix4fv(gp_view_mat_loc, 1, gl::FALSE, view_mat.as_ptr());
        gl::UniformMatrix4fv(gp_proj_mat_loc, 1, gl::FALSE, proj_mat.as_ptr());
        gl::UseProgram(cube_sp);
        gl::UniformMatrix4fv(cube_view_mat_location, 1, gl::FALSE, rot_mat_inv.inverse().as_ptr());
        gl::UniformMatrix4fv(cube_proj_mat_location, 1, gl::FALSE, proj_mat.as_ptr());
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

    /*********************** RENDERING LOOP *******************************/
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
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, gp_tex);
            gl::UseProgram(gp_sp);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            // Draw the title screen. Disable depth testing and enable 
            // alpha blending to do so.
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, title_screen_tex);
            gl::UseProgram(title_screen_sp);
            gl::BindVertexArray(string_vao);
            gl::Uniform4f(title_screen_sp_colour_loc, 1.0, 0.0, 1.0, 1.0);
            gl::DrawArrays(gl::TRIANGLES, 0, string_points as i32);
        }

        context.glfw.poll_events();

        /*********************** UPDATE GAME STATE **************************/
        // control keys
        let mut cam_moved = false;
        let mut move_to = math::vec3((0.0, 0.0, 0.0));
        let mut cam_yaw = 0.0; // y-rotation in degrees
        let mut cam_pitch = 0.0;
        let mut cam_roll = 0.0;
        match context.window.get_key(Key::A) {
            Action::Press | Action::Repeat => {
                move_to.v[0] -= cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::D) {
            Action::Press | Action::Repeat => {
                move_to.v[0] += cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::Q) {
            Action::Press | Action::Repeat => {
                move_to.v[1] += cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::E) {
            Action::Press | Action::Repeat => {
                move_to.v[1] -= cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::W) {
            Action::Press | Action::Repeat => {
                move_to.v[2] -= cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::S) {
            Action::Press | Action::Repeat => {
                move_to.v[2] += cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::Left) {
            Action::Press | Action::Repeat => {
                cam_yaw += cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_yaw = Versor::from_axis_deg(cam_yaw, up.v[0], up.v[1], up.v[2]);
                q = q_yaw * &q;
            }
            _ => {}
        }
        match context.window.get_key(Key::Right) {
            Action::Press | Action::Repeat => {
                cam_yaw -= cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_yaw = Versor::from_axis_deg(cam_yaw, up.v[0], up.v[1], up.v[2]);
                q = q_yaw * &q;
            }
            _ => {}
        }
        match context.window.get_key(Key::Up) {
            Action::Press | Action::Repeat => {
                cam_pitch += cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_pitch = Versor::from_axis_deg(cam_pitch, rgt.v[0], rgt.v[1], rgt.v[2]);
                q = q_pitch * &q;
            }
            _ => {}
        }
        match context.window.get_key(Key::Down) {
            Action::Press | Action::Repeat => {
                cam_pitch -= cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_pitch = Versor::from_axis_deg(cam_pitch, rgt.v[0], rgt.v[1], rgt.v[2]);
                q = q_pitch * &q;
            }
            _ => {}
        }
        match context.window.get_key(Key::Z) {
            Action::Press | Action::Repeat => {
                cam_roll -= cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_roll = Versor::from_axis_deg(cam_roll, fwd.v[0], fwd.v[1], fwd.v[2]);
                q = q_roll * &q;
            }
            _ => {}
        }
        match context.window.get_key(Key::C) {
            Action::Press | Action::Repeat => {
                cam_roll += cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_roll = Versor::from_axis_deg(cam_roll, fwd.v[0], fwd.v[1], fwd.v[2]);
                q = q_roll * &q;        
            }
            _ => {}
        }

        // update view matrix
        if cam_moved {
            // re-calculate local axes so can move fwd in dir cam is pointing
            rot_mat_inv = q.to_mat4();
            fwd = rot_mat_inv * math::vec4((0.0, 0.0, -1.0, 0.0));
            rgt = rot_mat_inv * math::vec4((1.0, 0.0,  0.0, 0.0));
            up  = rot_mat_inv * math::vec4((0.0, 1.0,  0.0, 0.0));

            cam_pos += math::vec3(fwd) * -move_to.v[2];
            cam_pos += math::vec3(up)  *  move_to.v[1];
            cam_pos += math::vec3(rgt) *  move_to.v[0];
            trans_mat_inv = Mat4::identity().translate(&cam_pos);

            view_mat = rot_mat_inv.inverse() * trans_mat_inv.inverse();
            unsafe {
                gl::UseProgram(gp_sp);
                gl::UniformMatrix4fv(gp_view_mat_loc, 1, gl::FALSE, view_mat.as_ptr());

                // Cube map view matrix has rotation, but not translation. It moves with the camera.
                gl::UseProgram(cube_sp);
                gl::UniformMatrix4fv(cube_view_mat_location, 1, gl::FALSE, rot_mat_inv.inverse().as_ptr());
            }
        }

        // Check whether the user signaled GLFW to close the window.
        match context.window.get_key(Key::Escape) {
            Action::Press | Action::Repeat => {
                context.window.set_should_close(true);
            }
            _ => {}
        }
        /********************** END UPDATE GAME STATE **************************/

        context.window.swap_buffers();
    }
    /******************** END RENDERING LOOP *******************************/
}
