#![allow(dead_code)]

use std::fs;
use std::io;
use std::io::Write;

mod vault;
mod logger;
mod argparse;

const PATH: &str = "/home/qwerty/.rustsafe";
const PASSWORDFILE: &str = "/home/qwerty/.rustsafe/dump.json";
const LOGFILE: &str = "/home/qwerty/.rustsafe/log";
const EXPORT: &str = "/home/qwerty/desktop/main.json";

fn main() {
    logger::start_logger(LOGFILE);

    if !fs::exists(PATH).unwrap() {
        match initialize_database() {
            Ok(()) => println!("[+] Database was created!"),
            Err(x) => panic!("[!] Error: {x}"),
        }
        log!("Database was created");
        return;
    }
}

fn initialize_database() -> io::Result<()> {
    fs::create_dir(PATH)?;
    let _ = fs::File::create(PASSWORDFILE)?;
    let _ = fs::File::create(LOGFILE)?;
    Ok(())
}

fn verify_master_password() {}

fn display_stored_credentials() {}

fn store_new_credential() {
    /* load the file 
     * add new record
     * dump
     * */
}

fn update_existing_credential() {
    /* load the file 
     * find the required record
     * change the stuff
     * change the hmac
     * then dump */
}

fn update_master_password() {
    /* load the file 
     * take new key
     * then change the hmac of each record 
     * then dump */
}

fn import_credentials_from_json() {}

fn export_credentials_to_json() {}

