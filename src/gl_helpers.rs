use gl;
use gl::types::{GLubyte};
use glfw;
use glfw::{Context};

use std::ffi::CStr;
use std::sync::mpsc::Receiver;

use logger::Logger;


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
    pub logger: Logger,
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
    // Initiate a logger.
    let logger = Logger::from(log_file);
    logger.restart();

    // Start GL context and O/S window using the GLFW helper library.
    logger.log(&format!("Starting GLFW"));
    logger.log(&format!("Using GLFW version {}", glfw::get_version_string()));

    // Start a GL context and OS window using the GLFW helper library.
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

    /*******************************************************/
    /* TODO: INSERT APPLE SPECIFIC GL STARTUP CODE HERE.   */
    /*******************************************************/
    // glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3);
    // glfw.window_hint(glfw::WindowHint::ContextVersionMinor(2);
    // glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    // glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    /*******************************************************/

    logger.log(&format!("Started GLFW successfully\n"));
    let maybe_glfw_window = glfw.create_window(
        640, 480, &format!("Metroid DEMO @ {:.2} FPS", 0.0), glfw::WindowMode::Windowed
    );
    let (mut window, events) = match maybe_glfw_window {
        Some(tuple) => tuple,
        None => {
            logger.log("Failed to create GLFW window");
            return Err(format!("Failed to create GLFW window."));
        }
    };

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
    logger.log(&format!("Renderer: {}", renderer));

    let version = glubyte_ptr_to_string(unsafe { gl::GetString(gl::VERSION) });
    println!("OpenGL version supported: {}", version);
    logger.log(&format!("OpenGL version supported: {}", version));

    Ok(GLContext {
        glfw: glfw, 
        window: window, 
        events: events,
        logger: logger,
        width: 640,
        height: 480,
        channel_depth: 3,
        running_time_seconds: 0.0,
        framerate_time_seconds: 0.0,
        frame_count: 0,
    })
}
