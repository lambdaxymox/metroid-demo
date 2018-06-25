use chrono::prelude::Utc;
use std::fs::{File, OpenOptions};
use std::io::Write;


pub struct Logger {
    log_file: String,
}

impl Logger {
    ///
    /// Start a new log file with the time and date at the top.
    ///
    fn new(log_file: &str) -> Logger {
        Logger {
            log_file: String::from(log_file),
        }
    }

    ///
    /// Start a new log file with the time and date at the top.
    ///
    pub fn restart(&self) -> bool {
        let file = File::create(&self.log_file);
        if file.is_err() {
            eprintln!(
                "ERROR: The GL_LOG_FILE log file {} could not be opened for writing.", self.log_file
            );

            return false;
        }

        let mut file = file.unwrap();

        let date = Utc::now();
        write!(file, "GL_LOG_FILE log. local time {}", date).unwrap();
        write!(file, "build version: ??? ?? ???? ??:??:??\n\n").unwrap();

        return true;
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

        return true;
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

        return true;
    }
}

impl<'a> From<&'a str> for Logger {
    fn from(log_file: &'a str) -> Logger {
        Logger::new(log_file)
    }
}