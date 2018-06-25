extern crate gl;
extern crate glfw;

mod gl_helpers;

use glfw::{Action, Context, Key};
use gl::types::{GLfloat, GLsizeiptr, GLvoid, GLuint};
use std::mem;
use std::ptr;


fn main() {
    let mut context = gl_helpers::start_gl("");
}
