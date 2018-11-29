use chrono::prelude::Utc;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::fmt;
use std::path::Path;


pub struct FileLogger {
    log_file: String,
}

impl FileLogger {
    ///
    /// Start a new log file with the time and date at the top.
    ///
    fn new(log_file: &str) -> FileLogger {
        FileLogger {
            log_file: String::from(log_file),
        }
    }

    ///
    /// Finish writing to a log. This function is used to place any final
    /// information in a log file before the logger goes out of scope.
    ///
    fn finalize(&self) -> bool {
        let file = OpenOptions::new().write(true).append(true).open(&self.log_file);
        if file.is_err() {
            eprintln!("ERROR: Could not open GL_LOG_FILE {} file for appending.", &self.log_file);
            return false;
        }

        let mut file = file.unwrap();
        let date = Utc::now();
        writeln!(file, "Logging finished at local time {}", date).unwrap();
        writeln!(file, "END LOG").unwrap();

        true
    }

    ///
    /// Start a new log file with the time and date at the top.
    ///
    pub fn restart(&self) -> bool {
        let file = File::create(&self.log_file);
        if file.is_err() {
            eprintln!(
                "ERROR: The OpenGL log file \"{}\" could not be opened for writing.", self.log_file
            );

            return false;
        }

        let mut file = file.unwrap();

        let date = Utc::now();
        writeln!(file, "OpenGL application log.\nStarted at local time {}", date).unwrap();
        writeln!(file, "build version: ??? ?? ???? ??:??:??\n\n").unwrap();

        true
    }

    ///
    /// Write a message to the log file.
    ///
    pub fn log(&self, message: &str) -> bool {
        let file = OpenOptions::new().write(true).append(true).open(&self.log_file);
        if file.is_err() {
            eprintln!("ERROR: Could not open GL_LOG_FILE {} file for appending.", &self.log_file);
            return false;
        }

        let mut file = file.unwrap();
        writeln!(file, "{}", message).unwrap();

        true
    }

    ///
    /// Write a message to the log file, and also write it to stderr.
    ///
    pub fn log_err(&self, message: &str) -> bool {
        let file = OpenOptions::new().write(true).append(true).open(&self.log_file);
        if file.is_err() {
            eprintln!("ERROR: Could not open GL_LOG_FILE {} file for appending.", &self.log_file);
            return false;
        }

        let mut file = file.unwrap();
        writeln!(file, "{}", message).unwrap();
        eprintln!("{}", message);

        true
    }
}

impl<'a> From<&'a str> for FileLogger {
    fn from(log_file: &'a str) -> FileLogger {
        FileLogger::new(log_file)
    }
}

impl Drop for FileLogger {
    fn drop(&mut self) {
        self.finalize();
    }
}

impl fmt::Write for FileLogger {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        match self.log(s) {
            true => Ok(()),
            false => Err(fmt::Error),
        }
    }
}

///
/// Initialize a file logger with the specified logging level.
///
pub fn init_with_level<P: AsRef<Path>>(
    log_file: P, level: log::Level) -> Result<(), log::SetLoggerError> {

    let logger = FileLogger::new(log_file, level);
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(level.to_level_filter());
    Ok(())
}

///
/// Initialize a file logger that logs all messages by default.
///
pub fn init<P: AsRef<Path>>(log_file: P) -> Result<(), log::SetLoggerError> {
    init_with_level(log_file, log::Level::Trace)
}


#[macro_export]
macro_rules! log {
    ($logger:expr, $format:expr) => {
        $logger.log($format);
    };
    ($logger:expr, $format:expr, $($arg:expr), *) => {
        $logger.log(&format!($format, $($arg,)*));
    };
}

#[macro_export]
macro_rules! log_err {
    ($logger:expr, $format:expr) => {
        $logger.log_err($format);
    };
    ($logger:expr, $format:expr, $($arg:expr), *) => {
        $logger.log_err(&format!($format, $($arg,)*));
    };    
}
