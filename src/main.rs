use std::fs;
use std::io;

mod vault;
mod logger;

const PATH: &str = "/home/qwerty/.rustsafe";
const PASSWORDFILE: &str = "/home/qwerty/.rustsafe/dump.json";
const EXPORT: &str = "/home/qwerty/desktop/main.json";

fn main() {
    if !fs::exists(PATH).unwrap() {
        // create_database       
    }
}

fn create_database() -> io::Result<()> {
    // create the folder and then the files   
    Ok(())
}

fn check_password() {}

fn view_saved_passwords() {}

fn add_new_password() {
    /* load the file 
     * add new record
     * dump
     * */
}

fn edit_exitisting_password() {
    /* load the file 
     * find the required record
     * change the stuff
     * change the hmac
     * then dump */
}

fn import_from_json_file() {}

fn export_to_json_file() {}

fn change_database_password() {
    /* load the file 
     * take new key
     * then change the hmac of each record 
     * then dump */
}
