#![allow(dead_code)]

use std::fs;
use std::io;
use std::io::Write;
use std::env;

mod vault;
mod logger;
mod argparse;

const PATH: &str = "/home/qwerty/.rustsafe";
const PASSWORDFILE: &str = "/home/qwerty/.rustsafe/dump.json";
const LOGFILE: &str = "/home/qwerty/.rustsafe/log";
const EXPORT: &str = "/home/qwerty/desktop/main.json";

type Commands = argparse::Commands;

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
    
    match argparse::parse_args(env::args()) {
        Some(cmd) => {
            match cmd {
                Commands::Add(x) => {
                    
                },
                _ => {},
            }
        },
        None => {
            return;
        }
    };
    
}

fn initialize_database() -> io::Result<()> {
    fs::create_dir(PATH)?;
    let _ = fs::File::create(PASSWORDFILE)?;
    let _ = fs::File::create(LOGFILE)?;
    Ok(())
}

fn display_stored_credentials() {}

fn store_new_credential(entry: String) {
    let mut data: Vec<String> = Vec::with_capacity(5);
    /* load the file 
     * add new record
     * dump
     * */
    print!("[+] Enter master password: ");
    let password: String = vault::fgets();
    
    print!("[+] Enter username for '{}': ", entry);
    data.push(vault::fgets());
    
    print!("[+] Enter password for '{}' (on empty creates a safe password of length 30) : ", entry);
    let pass: String = vault::fgets();
    if pass.is_empty() {
        data.push(vault::generate_rand_password(30));
    } else {
        data.push(pass);
    }

    print!("[+] Enter email for '{}' (optional): ", entry);
    data.push(vault::fgets());
    
    print!("[+] Enter note for '{}' (optional): ", entry);
    data.push(vault::fgets());

    let mut records = match vault::load(PASSWORDFILE, &password) {
        Some(x) => x,
        None => {
            println!("[!] Please create the database first!.\nTry 'rustsafe init' to create the database");
            return;
        },
    };

    records.push(vault::Record::new(&data, &password));
    
    vault::dump(&records, PASSWORDFILE, &password);
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

