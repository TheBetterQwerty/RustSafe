#![allow(dead_code)]

/* IMPORTS */
use std::fs::{File, OpenOptions};
use std::sync::Mutex;
use std::io::{Seek, SeekFrom, Write, Read};
use std::time::{SystemTime, UNIX_EPOCH};

/* GLOBAL VAR'S */
static LOGGER: Mutex<Option<File>> = Mutex::new(None);
const SIZE: usize = 300;
const BAN: &str = "You are banned for 5 minutes";
const BAN_TIME: u128 = 3_00_000; // 5 minutes

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

fn get_current_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

/* This function will return the time left until unban 
 * if it return None that means that last log isnt a BAN log 
 * OR user's ban time is up
 * */
pub fn get_elapsed_time() -> bool {
    let mut time_left: u128 = 0;
    let mut logger = LOGGER.lock().unwrap();
    if let Some(ref mut file) = *logger {
        let mut data = String::new();
        file.seek(SeekFrom::Start(0)).unwrap();
        let _ = file.read_to_string(&mut data);
        let logs: Vec<_> = data.split('\n').collect();
        if let Some(last_log) = logs.last() {
            if !last_log.contains(BAN) {
                return false;
            }
            /* 12345667 HELLO WORLD */
            let arg = last_log.split(' ').nth(0).unwrap(); // [ 12345667 , HELLO , WORLD ]
            let arg: u128 = arg.parse().expect("[!] Error parsing data!"); // 12345667 -> u128

            time_left = get_current_time() - arg;
        }
    }
    
    if time_left < BAN_TIME {
        return true;
    }

    return false;
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        check_file_length(); // checks for length
        let mut logger = LOGGER.lock().unwrap();
        if let Some(file) = *logger {
            writeln!(file, "{} {}", get_current_time(), $($arg)*).expect("[!] Error writting to log file!");
        }
    };
}
