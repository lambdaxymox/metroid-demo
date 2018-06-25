extern crate gl;
extern crate glfw;
extern crate chrono;

mod gl_helpers;
mod logger;

use glfw::{Action, Context, Key};
use gl::types::{GLfloat, GLsizeiptr, GLvoid, GLuint};
use std::mem;
use std::ptr;


fn main() {
    let mut context = gl_helpers::start_gl("gl.log");
}
