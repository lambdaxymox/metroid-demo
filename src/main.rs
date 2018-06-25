extern crate gl;
extern crate glfw;
extern crate chrono;

#[macro_use]
mod logger;

mod gl_helpers;
mod math;

use glfw::{Action, Context, Key};
use gl::types::{GLfloat, GLsizeiptr, GLvoid, GLuint};
use std::mem;
use std::ptr;
use std::process;

use gl_helpers as glh;
use math::{Mat4};


fn main() {
    let mut context = match glh::start_gl("gl.log") {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to Initialize OpenGL context. Got error:");
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    let points: [GLfloat; 9] = [ 
        0.5, -0.5, 0.0, 0.0, 0.5, 0.0, -0.5, -0.5, 0.0
    ];

    let colors: [GLfloat; 9] = [
        1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0
    ];

    let mut points_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut points_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (mem::size_of::<GLfloat>() * points.len()) as GLsizeiptr, 
            points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
    }
    assert!(points_vbo > 0);

    let mut colors_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut colors_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, colors_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (mem::size_of::<GLfloat>() * colors.len()) as GLsizeiptr, 
            colors.as_ptr() as *const GLvoid, gl::STATIC_DRAW
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

    /*************************** CAMERA MODEL *****************************/
    let near = 0.1;
    let far = 100.0;
    let fov = 67.0;
    let aspect = context.width as f32 / context.height as f32;
    let proj_mat = Mat4::perspective(fov, aspect, near, far);

    // View matrix components.
    let _cam_speed: GLfloat = 3.0;
    let cam_yaw_speed: GLfloat = 50.0;
    let mut cam_pos: [GLfloat; 3] = [0.0, 0.0, 2.0];
    let mut cam_yaw: GLfloat = 0.0;
    let mut trans_mat = Mat4::identity().translate(&math::vec3((-cam_pos[0], -cam_pos[1], -cam_pos[2])));
    let mut rot_mat = Mat4::identity().rotate_y_deg(-cam_yaw);
    let mut view_mat = rot_mat * trans_mat;

    /*************************** ****** ***** *****************************/

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

    unsafe {
        gl::UseProgram(shader_program);
        gl::UniformMatrix4fv(model_mat_location, 1, gl::FALSE, model_mat.as_ptr());
        gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
        gl::UniformMatrix4fv(proj_mat_location, 1, gl::FALSE, proj_mat.as_ptr());
    }

    unsafe {
        // Enable depth-testing.
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
        gl::FrontFace(gl::CCW);
    }

    while !context.window.should_close() {
        glh::update_timers(&mut context);
        glh::update_fps_counter(&mut context);

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Viewport(0, 0, context.width as i32, context.height as i32);
            
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);

            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        context.glfw.poll_events();

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
