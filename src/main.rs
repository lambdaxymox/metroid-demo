extern crate gl;
extern crate glfw;
extern crate chrono;

#[macro_use]
mod logger;

mod gl_helpers;

use glfw::{Action, Context, Key};
use gl::types::{GLfloat, GLsizeiptr, GLvoid, GLuint};
use std::mem;
use std::ptr;
use std::process;

use gl_helpers as glh;


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

    let mut points_vbo: GLuint = 0;
    unsafe {
        gl::GenBuffers(1, &mut points_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (mem::size_of::<GLfloat>() * points.len()) as GLsizeiptr, 
            points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
    }
    assert!(points_vbo > 0);

    let mut vao: GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::EnableVertexAttribArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
    }
    assert!(vao > 0);

    let shader_program = glh::create_program_from_files(
        &context, "shaders/metroid_demo.vert.glsl", "shaders/metroid_demo.frag.glsl"
    );

    let in_color_vec = [255.0/255.0, 20.0/255.0, 147.0/255.0];
    let in_color_location = unsafe {
        gl::GetUniformLocation(shader_program, "in_color".as_ptr() as *const i8)
    };
    assert!(in_color_location != -1);
    unsafe {
        gl::UseProgram(shader_program);
        gl::Uniform3f(in_color_location, in_color_vec[0], in_color_vec[1], in_color_vec[2]);
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
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);

            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        context.glfw.poll_events();
        context.window.swap_buffers();

        // Check whether the user signaled GLFW to close the window.
        match context.window.get_key(Key::Escape) {
            Action::Press | Action::Repeat => {
                context.window.set_should_close(true);
            }
            _ => {}
        }
    }
}
