/* IMPORTS */
use std::fs::{File, OpenOptions};
use std::sync::Mutex;
use std::io::{Seek, SeekFrom, Write, Read};
use std::time::{SystemTime, UNIX_EPOCH};

/* GLOBAL VAR'S */
pub static LOGGER: Mutex<Option<File>> = Mutex::new(None);
const SIZE: usize = 300;
const BAN: &str = "You are banned for 5 minutes";
const BAN_TIME: u128 = 3_00_000; // 5 minutes

pub fn start_logger(path: &str) -> Option<u128> {
    let file = match OpenOptions::new().read(true).write(true).create(true).open(path) {
        Ok(x) => x,
        Err(x) => panic!("[!] Error: {x}"),
    };
    
    let mut logger = LOGGER.lock().unwrap();
    *logger = Some(file);

    check_if_banned()
}

pub fn check_file() {
    let mut logger = LOGGER.lock().unwrap();
    if let Some(ref mut file) = *logger {
        let mut data: String = String::new();
        let _ = file.read_to_string(&mut data).unwrap();

        let mut lines: Vec<_> = data
            .split('\n')
            .filter(|x| !x.is_empty())
            .collect();

        if lines.len() >= SIZE {
            lines.drain(0..(lines.len() - SIZE + 1));

            file.set_len(0).unwrap();
            file.seek(SeekFrom::Start(0)).unwrap();
            write!(file, "{}", lines.join("\n")).unwrap();
        }
    }
}

pub fn get_current_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn give_ban() {
    let mut logger = LOGGER.lock().unwrap();
    if let Some(ref mut file) = *logger {
        let mut data = String::new();
        file.seek(SeekFrom::Start(0)).unwrap();
        file.read_to_string(&mut data).unwrap();
        
        let fails = data
            .split('\n')
            .rev()
            .take(5)
            .filter(|x| x.contains("Login Failed"))
            .count();

        if fails == 5 {
            writeln!(file, "{} {}", get_current_time(), BAN).expect("[!] Error writting to logfile");
            return;
        }

        writeln!(file, "{} Login Failed", get_current_time()).expect("[!] Error writting to logfile");
    }
}

fn check_if_banned() -> Option<u128> {
    let mut logger = LOGGER.lock().unwrap();
    if let Some(ref mut file) = *logger {
        let mut data = String::new();
        file.seek(SeekFrom::Start(0)).unwrap();
        file.read_to_string(&mut data).unwrap();
        
        let last_log = data
                .split('\n')
                .rev()
                .nth(0)
                .unwrap();
        
        if last_log.contains(BAN) {
            let lock_time: u128 = last_log
                .split(' ')
                .nth(0)
                .unwrap()
                .parse()
                .unwrap_or(0);

            let time_left = get_current_time() - lock_time;

            if time_left > BAN_TIME {
                return None;
            }
            
            return Some(time_left);
        }
    }

    None
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        logger::check_file();
        let mut logger = logger::LOGGER.lock().unwrap();
        if let Some(ref mut file) = *logger {
            let time = logger::get_current_time();
            writeln!(file, "{} {}", time, format!($($arg)*)).expect("[!] Error writting to log file!");
        }
    };
}
