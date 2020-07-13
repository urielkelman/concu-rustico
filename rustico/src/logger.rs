pub type LogFile = Option<Arc<Mutex<LineWriter<File>>>>;

use std::fs::{self, File};
use std::io::prelude::*;
use std::io::LineWriter;
use std::sync::{Arc, Mutex};
use std::error::Error;

pub fn create_logfile(filename: String) -> Result<LogFile, Box<dyn Error>>{
    let raw_file = File::create(filename)?;
    let file = LineWriter::new(raw_file);
    return Ok(Some(Arc::new(Mutex::new(file))));
}

pub fn debug(file: LogFile, message: String) -> std::io::Result<()>{
    if file.is_some(){
        file.unwrap().lock().unwrap().write_all(format!("DEBUG: {}\n", message).as_bytes())?;
    }
    Ok(())
}

pub fn info(file: LogFile, message: String) -> std::io::Result<()>{
    if file.is_some(){
        file.unwrap().lock().unwrap().write_all(format!("INFO: {}\n", message).as_bytes())?;
    }
    Ok(())
}

pub fn error(file: LogFile, message: String) -> std::io::Result<()>{
    if file.is_some(){
        file.unwrap().lock().unwrap().write_all(format!("ERROR: {}\n", message).as_bytes())?;
    }
    Ok(())
}
