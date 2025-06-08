#![allow(dead_code)]

use std::fs::{File, OpenOptions};
use std::sync::Mutex;
use std::io::{Seek, SeekFrom, Write, Read};
use std::time::{SystemTime, UNIX_EPOCH};

static LOGGER: Mutex<Option<File>> = Mutex::new(None);
const SIZE: usize = 300;
const BAN: &str = "You are banned for 5 minutes";

pub fn start_logger(path: &str) {
    let file = match OpenOptions::new().read(true).write(true).create(true).open(path) {
        Ok(x) => x,
        Err(x) => panic!("[!] Error: {x}"),
    };
    
    let mut logger = LOGGER.lock().unwrap();
    *logger = Some(file);
}

fn check_file_length() {
    let mut logger = LOGGER.lock().unwrap();
    if let Some(ref mut file) = *logger {
        let mut data: String = String::new();
        file.seek(SeekFrom::Start(0)).unwrap();

        let _ = file.read_to_string(&mut data);
        let n = data.chars().filter(|c| *c == '\n').count();

        if n >= SIZE {
            let mut x: Vec<_> = data.split('\n').collect();
            let _ = x.remove(0);
            
            file.set_len(0).unwrap();
            file.seek(SeekFrom::Start(0)).unwrap();
            write!(file, "{}", x.join("\n")).unwrap();
        }
    }
}

fn get_current_time() -> SystemTime {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
}

fn get_elapsed_time() -> Option<u64> {
    let mut logger = LOGGER.lock().unwrap();
    if let Some(ref mut file) = *logger {
        let mut data = String::new();
        file.seek(SeekFrom::Start(0)).unwrap();
        let _ = file.read_to_string(&mut data);
        let logs: Vec<_> = data.split('\n').collect();
        if let Some(last_log) = logs.last() {
            if !last_log.contains(BAN) {
                return None;
            }

            let args: Vec<_> = last_log.split(' ').collect();
            let current_time = get_current_time();
            
        }
    }

    Some(0)
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        check_file_length(); // checks for length
        let mut logger = LOGGER.lock().unwrap();
        if let Some(file) = *logger {
            writeln!(file, "{} {}", ,$($arg)*).expect("[!] Error writting to log file!");
        }
    };
}
