pub type LogFile = Arc<Mutex<Option<LineWriter<File>>>>;

use std::fs::File;
use std::io::prelude::*;
use std::io::LineWriter;
use std::sync::{Arc, Mutex};
use std::error::Error;

pub fn create_logfile(filename: String) -> Result<LogFile, Box<dyn Error>>{
    let raw_file = File::create(filename)?;
    let file = LineWriter::new(raw_file);
    return Ok(Arc::new(Mutex::new(Some(file))));
}

fn log(file: LogFile, message: String, level: String) -> std::io::Result<()>{
    if file.lock().unwrap().is_some(){
        {
            let mut line_writer_locked = file.lock().unwrap();
            line_writer_locked.as_mut().unwrap().write_all(format!("{}: {}\n", level, message).as_bytes())?;
            line_writer_locked.as_mut().unwrap().flush()?;
        }
    }
    Ok(())
}

pub fn debug(file: LogFile, message: String) -> std::io::Result<()>{
    return log(file, message, "DEBUG".to_string());
}

pub fn info(file: LogFile, message: String) -> std::io::Result<()>{
    return log(file, message, "INFO".to_string());

}

pub fn error(file: LogFile, message: String) -> std::io::Result<()>{
    return log(file, message, "ERROR".to_string());
}
