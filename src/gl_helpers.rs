use gl;
use gl::types::{GLubyte};
use glfw;
use glfw::{Context};

use std::ffi::CStr;
use std::sync::mpsc::Receiver;


#[inline]
pub fn glubyte_ptr_to_string(cstr: *const GLubyte) -> String {
    unsafe {
        CStr::from_ptr(cstr as *const i8).to_string_lossy().into_owned()
    }
}

///
/// A record for storing all the OpenGL state needed on the application side
/// of the graphics application in order to manage OpenGL and GLFW.
///
pub struct GLContext {
    pub glfw: glfw::Glfw,
    pub window: glfw::Window,
    pub events: Receiver<(f64, glfw::WindowEvent)>,
    pub width: u32,
    pub height: u32,
    pub channel_depth: u32,
    pub running_time_seconds: f64,
    pub framerate_time_seconds: f64,
    pub frame_count: u32,
}

///
/// Initialize a new OpenGL context and start a new GLFW window. 
///
pub fn start_gl(log_file: &str) -> Result<GLContext, String> {
    // Start a GL context and OS window using the GLFW helper library.
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

    let (mut window, events) = glfw.create_window(
        640, 480, &format!("Metroid DEMO @ {:.2} FPS", 0.0), glfw::WindowMode::Windowed
    )
    .expect("Failed to create GLFW window.");

    window.make_current();
    window.set_key_polling(true);
    window.set_size_polling(true);
    window.set_refresh_polling(true);
    window.set_size_polling(true);

    // Load the OpenGl function pointers.
    gl::load_with(|symbol| { window.get_proc_address(symbol) as *const _ });

    // Get renderer and version information.
    let renderer = glubyte_ptr_to_string(unsafe { gl::GetString(gl::RENDERER) });
    println!("Renderer: {}", renderer);
    
    let version = glubyte_ptr_to_string(unsafe { gl::GetString(gl::VERSION) });
    println!("OpenGL version supported: {}", version);


    Ok(GLContext {
        glfw: glfw, 
        window: window, 
        events: events,
        width: 640,
        height: 480,
        channel_depth: 3,
        running_time_seconds: 0.0,
        framerate_time_seconds: 0.0,
        frame_count: 0,
    })
}
