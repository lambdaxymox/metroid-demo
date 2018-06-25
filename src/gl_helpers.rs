use gl;
use gl::types::{GLchar, GLenum, GLubyte, GLuint};
use glfw;
use glfw::{Context};

use std::ffi::CStr;
use std::fs::File;
use std::io::{Read, BufReader};
use std::sync::mpsc::Receiver;
use std::ptr;
use std::fmt;

use logger::Logger;


// 256 Kilobytes.
const MAX_SHADER_LENGTH: usize = 262144;


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
    log!(logger, "Starting GLFW");
    log!(logger, "Using GLFW version {}", glfw::get_version_string());

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

    log!(logger, "Started GLFW successfully");
    let maybe_glfw_window = glfw.create_window(
        640, 480, &format!("Metroid DEMO @ {:.2} FPS", 0.0), glfw::WindowMode::Windowed
    );
    let (mut window, events) = match maybe_glfw_window {
        Some(tuple) => tuple,
        None => {
            log!(logger, "Failed to create GLFW window");
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
    log!(logger, "Renderer: {}", renderer);

    let version = glubyte_ptr_to_string(unsafe { gl::GetString(gl::VERSION) });
    println!("OpenGL version supported: {}", version);
    log!(logger, "OpenGL version supported: {}", version);

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

///
/// Updates the timers in a GL context. It returns the elapsed time since the last call to
/// `update_timers`.
///
#[inline]
pub fn update_timers(context: &mut GLContext) -> f64 {
    let current_seconds = context.glfw.get_time();
    let elapsed_seconds = current_seconds - context.running_time_seconds;
    context.running_time_seconds = current_seconds;

    elapsed_seconds
}

///
/// Update the framerate and display in the window titlebar.
///
#[inline]
pub fn update_fps_counter(context: &mut GLContext) {     
    let current_time_seconds = context.glfw.get_time();
    let elapsed_seconds = current_time_seconds - context.framerate_time_seconds;
    if elapsed_seconds > 0.5 {
        context.framerate_time_seconds = current_time_seconds;
        let fps = context.frame_count as f64 / elapsed_seconds;
        context.window.set_title(&format!("Metroid DEMO @ {:.2} FPS", fps));
        context.frame_count = 0;
    }

    context.frame_count += 1;
}


pub fn parse_shader(file_name: &str, shader_str: &mut [u8], max_len: usize) -> Result<usize, String> {
    shader_str[0] = 0;
    let file = match File::open(file_name) {
        Ok(val) => val,
        Err(_) => {
            return Err(format!("ERROR: opening file for reading: {}\n", file_name));
        }
    };

    let mut reader = BufReader::new(file);
    let bytes_read = match reader.read(shader_str) {
        Ok(val) => val,
        Err(_) => {
            return Err(format!("ERROR: reading shader file {}\n", file_name));
        }
    };

    // Append \0 character to end of the shader string to mark the end of a C string.
    shader_str[bytes_read] = 0;

    Ok(bytes_read)
}

///
/// A record containing all the relevant compilation log information for a
/// given GLSL shader compiled at run time.
///
pub struct ShaderLog {
    index: GLuint,
    log: String,
}

impl fmt::Display for ShaderLog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Shader info log for GL index {}:", self.index).unwrap();
        writeln!(f, "{}", self.log)
    }
}

///
/// Query the shader information log generated during shader compilation from
/// OpenGL.
/// 
pub fn shader_info_log(shader_index: GLuint) -> ShaderLog {
    let mut actual_length = 0;
    let mut raw_log: [i8; 2048] = [0; 2048];
    unsafe {
        gl::GetShaderInfoLog(shader_index, raw_log.len() as i32, &mut actual_length, &mut raw_log[0]);
    }
    
    let mut log = String::new();
    for i in 0..actual_length as usize {
        log.push(raw_log[i] as u8 as char);
    }

    ShaderLog { index: shader_index, log: log }
}

pub fn create_shader(context: &GLContext, file_name: &str, shader: &mut GLuint, gl_type: GLenum) -> bool {
    log!(context.logger, "Creating shader from {}...\n", file_name);

    let mut shader_string = vec![0; MAX_SHADER_LENGTH];
    let bytes_read = match parse_shader(file_name, &mut shader_string, MAX_SHADER_LENGTH) {
        Ok(val) => val,
        Err(st) => {
            log_err!(context.logger, &st);
            return false;
        }
    };

    if bytes_read >= (MAX_SHADER_LENGTH - 1) {
        log!(context.logger,
            "WARNING: The shader was truncated because the shader code 
            was longer than MAX_SHADER_LENGTH {} bytes.", MAX_SHADER_LENGTH
        );
    }

    *shader = unsafe { gl::CreateShader(gl_type) };
    let p = shader_string.as_ptr() as *const GLchar;
    unsafe {
        gl::ShaderSource(*shader, 1, &p, ptr::null());
        gl::CompileShader(*shader);
    }

    // Check for shader compile errors.
    let mut params = -1;
    unsafe {
        gl::GetShaderiv(*shader, gl::COMPILE_STATUS, &mut params);
    }

    if params != gl::TRUE as i32 {
        log_err!(context.logger, "ERROR: GL shader index {} did not compile\n", *shader);
        log_err!(context.logger, "{}", shader_info_log(*shader));
        
        return false;
    }
    log!(context.logger, "Shader compiled with index {}\n", *shader);
    
    return true;
}


///
/// A record containing all the relevant compilation log information for a
/// given GLSL shader program compiled at run time.
///
pub struct ProgramLog {
    index: GLuint,
    log: String,
}

impl fmt::Display for ProgramLog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Program info log for GL index {}:", self.index).unwrap();
        writeln!(f, "{}", self.log)
    }
}

///
/// Query the shader program information log generated during shader compilation 
/// from OpenGL.
/// 
pub fn program_info_log(index: GLuint) -> ProgramLog {
    let mut actual_length = 0;
    let mut raw_log = [0 as i8; 2048];
    unsafe {
        gl::GetProgramInfoLog(index, raw_log.len() as i32, &mut actual_length, &mut raw_log[0]);
    }
    
    let mut log = String::new();
    for i in 0..actual_length as usize {
        log.push(raw_log[i] as u8 as char);
    }

    ProgramLog { index: index, log: log }
}

///
/// Validate a shader program.
///
pub fn is_program_valid(logger: &Logger, sp: GLuint) -> bool {
    let mut params = -1;
    unsafe {
        gl::ValidateProgram(sp);
        gl::GetProgramiv(sp, gl::VALIDATE_STATUS, &mut params);
    }

    if params != gl::TRUE as i32 {
        log_err!(logger, "Program {} GL_VALIDATE_STATUS = GL_FALSE\n", sp);
        log_err!(logger, "{}", program_info_log(sp));
        
        return false;
    }

    log!(logger, "Program {} GL_VALIDATE_STATUS = {}\n", sp, params);
    
    return true;
}

///
/// Compile and link a shader program.
///
pub fn create_program(context: &GLContext, vertex_shader: GLuint, fragment_shader: GLuint, program: &mut GLuint) -> bool {
    unsafe {
        *program = gl::CreateProgram();
        log!(context.logger, "Created programme {}. attaching shaders {} and {}...\n", 
            program, vertex_shader, fragment_shader
        );
        gl::AttachShader(*program, vertex_shader);
        gl::AttachShader(*program, fragment_shader);

        // Link the shader programme. If binding input attributes do that before linking.
        gl::LinkProgram(*program);
        let mut params = -1;
        gl::GetProgramiv(*program, gl::LINK_STATUS, &mut params);
        if params != gl::TRUE as i32 {
            log_err!(context.logger, "ERROR: could not link shader programme GL index {}\n", *program);
            log_err!(context.logger, "{}", program_info_log(*program));
        
            return false;
        }
        is_program_valid(&context.logger, *program);
        // Delete shaders here to free memory
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        return true;
    }
}

///
/// Compile and link a shader program directly from the files.
///
pub fn create_program_from_files(context: &GLContext, vert_file_name: &str, frag_file_name: &str) -> GLuint {
    let mut vertex_shader = 0;
    let mut fragment_shader = 0;
    let mut program: GLuint = 0;
    
    create_shader(context, vert_file_name, &mut vertex_shader, gl::VERTEX_SHADER);
    create_shader(context, frag_file_name, &mut fragment_shader, gl::FRAGMENT_SHADER);
    create_program(context, vertex_shader, fragment_shader, &mut program);
    
    program
}
