extern crate gl;
extern crate glfw;
extern crate chrono;
extern crate stb_image;

#[macro_use]
mod logger;

mod gl_helpers;
mod math;

use glfw::{Action, Context, Key};
use gl::types::{GLchar, GLfloat, GLint, GLsizeiptr, GLvoid, GLuint};

use stb_image::image;
use stb_image::image::LoadResult;

use std::mem;
use std::ptr;
use std::process;

use gl_helpers as glh;
use math::{Mat4, Versor};

const GL_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;
const GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;


fn create_ground_plane_shaders(view_mat: &Mat4, proj_mat: &Mat4) -> (GLuint,  GLint, GLint) {
    // Here I used negative y from the buffer as the z value so that it was on
    // the floor but also that the 'front' was on the top side. also note how I
    // work out the texture coordinates, st, from the vertex point position.
    let mut gp_vs_str = vec![0; 1024];
    if glh::parse_shader("shaders/gp_vs.glsl", &mut gp_vs_str).is_err() {
        panic!("Failed to parse ground plane vertex shader file.");
    }

    let mut gp_fs_str = vec![0; 1024];
    if glh::parse_shader("shaders/gp_fs.glsl", &mut gp_fs_str).is_err() {
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

fn load_texture(file_name: &str, tex: &mut GLuint) -> bool {
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
        //gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        //gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
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
    let mut context = match glh::start_gl("gl.log") {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to Initialize OpenGL context. Got error:");
            eprintln!("{}", e);
            process::exit(1);
        }
    };
    /*
    let triangle_points: [GLfloat; 9] = [ 
        0.5, -0.5, 1.0, 0.0, 0.5, 1.0, -0.5, -0.5, 1.0
    ];

    let triangle_colors: [GLfloat; 9] = [
        1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0
    ];
    */
    
    let ground_plane_points: [GLfloat; 18] = [
         10.0,  10.0, 0.0, -10.0,  10.0, 0.0, -10.0, -10.0, 0.0, 
        -10.0, -10.0, 0.0,  10.0, -10.0, 0.0,  10.0,  10.0, 0.0
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
    /*
    let mut points_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut points_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (mem::size_of::<GLfloat>() * triangle_points.len()) as GLsizeiptr, 
            triangle_points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
    }
    assert!(points_vbo > 0);

    let mut colors_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut colors_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, colors_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (mem::size_of::<GLfloat>() * triangle_colors.len()) as GLsizeiptr, 
            triangle_colors.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
    }
    assert!(colors_vbo > 0);

    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(0);

        gl::BindBuffer(gl::ARRAY_BUFFER, colors_vbo);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(1);
    }
    assert!(vao > 0);
    

    let shader_program = glh::create_program_from_files(
        &context, "shaders/metroid_demo.vert.glsl", "shaders/metroid_demo.frag.glsl"
    );
    */
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

    /*************************** ****** ***** *****************************/
    /*
    let model_mat = Mat4::identity();

    let model_mat_location = unsafe {
        gl::GetUniformLocation(shader_program, "model".as_ptr() as *const i8)
    };
    assert!(model_mat_location > -1);

    let view_mat_location = unsafe {
        gl::GetUniformLocation(shader_program, "view".as_ptr() as *const i8)
    };
    assert!(view_mat_location > -1);

    let proj_mat_location = unsafe {
        gl::GetUniformLocation(shader_program, "proj".as_ptr() as *const i8)
    };
    assert!(proj_mat_location > -1);
    */

    // Load the shader program for the ground plane.
    let (gp_sp, gp_view_mat_loc, gp_proj_mat_loc) = create_ground_plane_shaders(&view_mat, &proj_mat);

    // Texture for the ground plane.
    let mut gp_tex = 0;
    load_texture("assets/tile2-diamonds256x256.png", &mut gp_tex);
    assert!(gp_tex > 0);
    
    unsafe {
        gl::UseProgram(gp_sp);
        gl::UniformMatrix4fv(gp_view_mat_loc, 1, gl::FALSE, view_mat.as_ptr());
        gl::UniformMatrix4fv(gp_proj_mat_loc, 1, gl::FALSE, proj_mat.as_ptr());
    }

    /*
    unsafe {
        gl::UseProgram(shader_program);
        gl::UniformMatrix4fv(model_mat_location, 1, gl::FALSE, model_mat.as_ptr());
        gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
        gl::UniformMatrix4fv(proj_mat_location, 1, gl::FALSE, proj_mat.as_ptr());
    }
    */

    unsafe {
        // Enable depth-testing.
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
        gl::FrontFace(gl::CCW);
    }

    while !context.window.should_close() {
        let elapsed_seconds = glh::update_timers(&mut context);
        glh::update_fps_counter(&mut context);

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Viewport(0, 0, context.width as i32, context.height as i32);
            
            // Draw the ground plane.
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, gp_tex);
            gl::UseProgram(gp_sp);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            // Draw the triangle above the ground plane.
            //gl::UseProgram(shader_program);
            //gl::BindVertexArray(vao);
            //gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        context.glfw.poll_events();

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
            }
        }

        // Check whether the user signaled GLFW to close the window.
        match context.window.get_key(Key::Escape) {
            Action::Press | Action::Repeat => {
                context.window.set_should_close(true);
            }
            _ => {}
        }

        context.window.swap_buffers();
    }
}
