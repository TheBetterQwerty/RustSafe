use std::{
    fs::File,
    io::{Read, Write},
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH}
};

#[derive(Debug, Clone, PartialEq)]
pub enum LogType {
    ERROR,
    DEBUG,
    BAN(u128),
    INVALID,
    INFO
}

pub static RW_LOG_FILE: OnceLock<File> = OnceLock::new();
pub const BAN_TIME: u128 = 5 * 60 * 1000;
const MAX_FAILS: usize = 5;
const MAX_LOGS: usize = 500;

#[macro_export]
macro_rules! log {
    ($debug: expr) => {{
        use std::fs::OpenOptions;

        {
            let file = OpenOptions::new()
                .read(true)     // for reading last logs
                .append(true)   // appending only
                .create(true)   // create if not exists
                .open($debug);

            if $crate::logger::RW_LOG_FILE.get().is_none() {
                let owned_file = file.unwrap();
                let _ = $crate::logger::RW_LOG_FILE.set(owned_file);
            }
        }

        let last_logs = match $crate::logger::get_last_logs(5usize, $debug) {
            Some(x) => x,
            None => Vec::new()
        };

        $crate::logger::ban_if_invalid(last_logs) // false if banned
    }};

    ($type:ident, $debug:expr) => {{
        use std::time::{SystemTime, UNIX_EPOCH};
        use std::io::Write;

        let time = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(time) => time.as_millis(),
            Err(_) => panic!("[!] Error: SytemTime Before UNIX_EPOCH"),
        };
        let log_type = $crate::logger::LogType::$type;

        match $crate::logger::RW_LOG_FILE.get() {
            Some(ref mut file) => {
                let _ = writeln!(file, "{} {:?} {}", time, log_type, $debug);
            },
            None => {},
        }
    }};
}

fn give_ban() {
    let time = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(time) => time.as_millis(),
        Err(_) => panic!("[!] Error: SytemTime Before UNIX_EPOCH"),
    };

    if let Some(ref mut file) = RW_LOG_FILE.get() {
        let _ = writeln!(file, "{} BAN User Banned for {} milliseconds", time, BAN_TIME);
    }

    println!(
        "[?] You have been banned for {} minutes.\nPlease wait until your ban expires before trying again.",
        (BAN_TIME / (1000 * 60))
    );
}

pub fn ban_if_invalid(logs: Vec<LogType>) -> bool {
    // false if banned
    let i = logs.iter().filter(|x| **x == LogType::INVALID).count();
    if i == MAX_FAILS {
        give_ban();
        return false;
    }

    let last_log = logs.last().unwrap_or(&LogType::DEBUG);
    if let Some(time) = time_till_unban(last_log) {
        println!(
            "[?] You are still banned. Time remaining: {} minutes and {} seconds.",
            (time / (1000 * 60)),
            (time / 1000) % 60
        );
        return false;
    }

    true
}

pub fn get_last_logs(n: usize, file: &str) -> Option<Vec<LogType>> {
    let mut buffer = String::new();
    if let Some(ref mut file) = RW_LOG_FILE.get() {
        file.read_to_string(&mut buffer).unwrap();
    }

    let mut str_logs: Vec<_> = buffer.lines().collect();

    let logs: Vec<_> = str_logs
        .iter()
        .map(|x| {
            if x.contains("ERROR") {
                return LogType::ERROR;
            }
            if x.contains("DEBUG") {
                return LogType::DEBUG;
            }
            if x.contains("BAN") {
                if let Some(time) = x.split_whitespace().nth(0) {
                    if let Ok(_t) = time.parse::<u128>() {
                        return LogType::BAN(_t);
                    }
                }
            }
            if x.contains("INFO") {
                return LogType::INFO;
            }
            LogType::INVALID
        })
        .collect();

    let len = logs.len();
    if len == 0 {
        return None;
    }

    if len > MAX_LOGS {
        let n_elements = len - MAX_LOGS;
        str_logs.drain(0..n_elements);
        if let Ok(mut file) = File::create(file) {
            for log in str_logs {
                let _ = writeln!(file, "{}", log);
            }
        }
    }

    let start = len.saturating_sub(n);
    Some(logs[start..].to_vec())
}

pub fn time_till_unban(log: &LogType) -> Option<u128> {
    if let LogType::BAN(time) = log {
        let max_ban_time = time + BAN_TIME;
        let current_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(time) => time.as_millis(),
            Err(x) => panic!("[!] Error: {x}")
        };

        return max_ban_time.checked_sub(current_time);
    }
    None
}
