/* Imports */
use std::{fs, io::{self, Write}, env};
use rpassword;

/* Modules */
mod vault;
mod logger;
mod argparse;

/* Constants */
const PATH: &str = "/home/qwerty/.rustsafe";
const PASSWORDFILE: &str = "/home/qwerty/.rustsafe/dump.json";
const LOGFILE: &str = "/home/qwerty/.rustsafe/log";
const EXPORTFILE: &str = "/home/qwerty/desktop/export.json";

type Commands = argparse::Commands;

fn main() {
    match argparse::parse_args(env::args()) {
        Some(cmd) => {
            match cmd {
                Commands::Init => match initialize_database() {
                    Ok(()) => println!("[+] Database created successfully"),
                    Err(x) => println!("[!] Error: {x}")
                },

                Commands::Generate(size) => println!("Generated Password -> {}", vault::generate_rand_password(size)),  

                _ => {

                    if !fs::exists(PATH).unwrap() {
                        println!("[!] Database isn't created.\nTry '{} init' to create a database", 
                            env::args()
                                .nth(0)
                                .unwrap_or("".to_string())
                        );
                        return;
                    }
                    
                    match logger::start_logger(LOGFILE) {
                        Some(milli) => {
                            let time: f64 = ((milli as f64) / 1000.0) / 60.0 ;
                            println!("[+] You are banned for {} minutes", time);
                            return;
                        },
                        None => {}
                    }

                    match cmd {
                        Commands::Add(entry) => store_new_credential(entry),
                        Commands::Get(entry) => display_stored_credentials(Some(entry)),
                        Commands::List => display_stored_credentials(None),
                        Commands::Edit(entry) => update_existing_credential(entry),
                        Commands::Delete(entry) => remove_existing_credential(entry),
                        Commands::Passwd => update_master_password(),
                        Commands::Import(path) => import_credentials_from_json(path),
                        Commands::Export => export_credentials_to_json(),
                        _ => {},
                    }
                }
            }
        },
        None => {},
    };
}

fn initialize_database() -> io::Result<()> {
    fs::create_dir(PATH)?;
    let _ = fs::File::create(PASSWORDFILE)?;
    let _ = fs::File::create(LOGFILE)?;
    println!("[+] Database created successfully!");

    log!("Database created successfully");
    Ok(())
}

fn display_stored_credentials(entry: Option<String>) {
    let password: String = rpassword::prompt_password("[+] Enter master password: ").unwrap();
    
    let records: Vec<vault::Record> = match vault::load(PASSWORDFILE, &password) {
        Ok(y) => match y {
            Some(x) => x,
            None => {
                println!("[!] No records were found!.\nTry 'rustsafe add' to create a new record");
                return;
            } },
        Err(err) => {
            if err.contains("[!] Error decrypting message") {
                println!("[!] Incorrect Password");
                logger::give_ban();
            }
            return;
        }
    };
    
    if let None = entry {        // if list is called
        vault::record_fmt(vault::RecordPrint::VECTOR(records));
        return;
    }
    
    let search = entry.unwrap_or(String::from(""));
    let mut found = false;
    
    for record in records {
        if record.entry().contains(&search) || record.username().contains(&search) {
            vault::record_fmt(vault::RecordPrint::RECORD(record));
            found = true;
            break;
        }
        
        if let Some(_email) = record.email() {
            if _email.contains(&search) {
                vault::record_fmt(vault::RecordPrint::RECORD(record));
                found = true;
                break;
            }
        }

        if let Some(note) = record.note() {
            if note.contains(&search) {
                vault::record_fmt(vault::RecordPrint::RECORD(record));
                found = true;
                break;
            }
        }
    }

    if !found {
        println!("[!] Record with '{}' doesn't exists", search);
        return;
    }
    
    log!("Records were viewed");
}

fn store_new_credential(entry: String) {
    let mut data: Vec<String> = vec![entry.clone()];

    let password: String = rpassword::prompt_password("[+] Enter master password: ").unwrap();

    let mut records: Vec<vault::Record> = match vault::load(PASSWORDFILE, &password) {
        Ok(y) => match y {
            Some(x) => x,
            None => Vec::new(),
        },
        Err(err) => {
            if err.contains("[!] Error decrypting message") {
                println!("[!] Incorrect Password");
                logger::give_ban();
            }
            return;
        }
    };

    print!("[+] Enter username for '{}': ", entry);
    data.push(vault::fgets());
    
    print!("[+] Enter password for '{}' (on empty creates a safe password of length 30) : ", entry);
    let mut pass: String = vault::fgets();
    if pass.is_empty() {
        pass = vault::generate_rand_password(30);
        println!("Generated password -> {}", pass);
    }
    data.push(pass);

    print!("[+] Enter email for '{}' (optional): ", entry);
    data.push(vault::fgets());
    
    print!("[+] Enter note for '{}' (optional): ", entry);
    data.push(vault::fgets());

    records.push(vault::Record::new(&data, &password));
    
    vault::dump(&records, PASSWORDFILE, &password);
    
    println!("[+] Credentials was stored into the database!");
    log!("New record was added to the database");
}

fn update_existing_credential(search: String) {
    let password: String = rpassword::prompt_password("[+] Enter master password: ").unwrap();
    
    let mut records: Vec<vault::Record> = match vault::load(PASSWORDFILE, &password) {
        Ok(y) => match y {
            Some(x) => x,
            None => {
                println!("[!] No records were found!.\nTry 'rustsafe add' to create a new record");
                return;
            }
        },
        Err(err) => {
            if err.contains("[!] Error decrypting message") {
                println!("[!] Incorrect Password");
                logger::give_ban();
            }
            return;
        }
    };
    
    let mut req_record: Option<(usize, &vault::Record)> = None;

    for (idx, record) in records.iter().enumerate() {
        if record.username().contains(&search) || (*record).entry().contains(&search) {
            req_record = Some((idx, record));
            break;
        }

        if let Some(_email) = record.email() {
            if _email.contains(&search) {
                req_record = Some((idx, record));
                break;
            }
        }

        if let Some(_note) = record.note() {
            if _note.contains(&search) {
                req_record = Some((idx, record));
                break;
            }
        }
    }

    if let None = req_record {
        println!("[!] No Records were found with that phrase '{}'", search);
        log!("Password updation no password's were found with the phrase '{}'", search);
        return;
    }
    
    let (idx, record) = req_record.unwrap();
    vault::record_fmt(vault::RecordPrint::RECORD(record.clone()));
    
    print!("[+] Do you want to change this record ? (Y/n) : ");
    let choice = vault::fgets().to_lowercase();
    
    if !choice.is_empty() && choice.starts_with('n') {
        println!("[#] Record Wasnt Updated!");
        return;
    }

    {
        let mut data: Vec<String> = Vec::new();
        print!("[+] Enter new username for '{}' (optional): ", (*record).entry());
        let _u = vault::fgets();
        if _u.is_empty() { data.push((*record).username()) } else { data.push(_u) }

        print!("[+] Enter new password for '{}' (optional): ", (*record).entry());
        let _p = vault::fgets();
        if _p.is_empty() { data.push((*record).password()) } else { data.push(_p) }

        print!("[+] Enter new email for '{}' (optional): ", (*record).entry());
        let _e = vault::fgets();
        if _e.is_empty() { 
            if let Some(_email) = (*record).email() {
                data.push(_email);
            } else {
                data.push(_e);              // just send the "" new function will convert it to None
            }
        } else {
            data.push(_e);
        }

        print!("[+] Enter new note for '{}' (optional): ", (*record).entry());
        let _n = vault::fgets();
        if _n.is_empty() { 
            if let Some(_note) = (*record).note() {
                data.push(_note);
            } else {
                data.push(_n);               // just send the "" new function will convert it to None
            }
        } else {
            data.push(_n);
        }

        records[idx] = vault::Record::new(&data, &password);

        println!("[+] Credentials was updated sucessfully");

        vault::dump(&records, PASSWORDFILE, &password);

        log!("Credentials was updated with the phrase '{}'", search);
    }   
}

fn update_master_password() {
    let password: String = rpassword::prompt_password("[+] Enter master password: ").unwrap();

    let records: Vec<vault::Record> = match vault::load(PASSWORDFILE, &password) {
        Ok(y) => match y {
            Some(x) => x,
            None => {
                println!("[!] No records were found!.\nTry 'rustsafe add' to create a new record");
                return;
            }
        },
        Err(err) => {
            if err.contains("[!] Error decrypting message") {
                println!("[!] Incorrect Password");
                logger::give_ban();
            }
            return;
        }
    };
    
    let passwd: String = rpassword::prompt_password("[+] Enter new master password: ").unwrap();
    let _password: String = rpassword::prompt_password("[+] Enter new master password again: ").unwrap();
 
    if passwd != _password {
        println!("[!] Passwords doesn't match!");
        return;
    }
    
    let mut new_records: Vec<vault::Record> = Vec::new();
    for record in records {
        let mut data = vec![record.entry(), record.username(), record.password()];

        if let Some(email) = record.email() {
            data.push(email);
        } else {
            data.push("".to_owned());
        }

        if let Some(note) = record.note() {
            data.push(note);
        } else {
            data.push("".to_owned());
        }

        new_records.push(vault::Record::new(&data, &_password));
    }

    vault::dump(&new_records, PASSWORDFILE, &_password);

    println!("[+] Master password was changed successfully!");
    log!("Master password was changed");
}

fn remove_existing_credential(search: String) {
    let password: String = rpassword::prompt_password("[+] Enter master password: ").unwrap();
    
    let mut records: Vec<vault::Record> = match vault::load(PASSWORDFILE, &password) {
        Ok(y) => match y {
            Some(x) => x,
            None => {
                println!("[!] No records were found!.\nTry 'rustsafe add' to create a new record");
                return;
            }
        },
        Err(err) => {
            if err.contains("[!] Error decrypting message") {
                println!("[!] Incorrect Password");
                logger::give_ban();
            }
            return;
        }
    };
    
    let mut req_record: Option<(usize, &vault::Record)> = None;

    for (idx, record) in records.iter().enumerate() {
        if record.username().contains(&search) || (*record).entry().contains(&search) {
            req_record = Some((idx, record));
            break;
        }

        if let Some(_email) = record.email() {
            if _email.contains(&search) {
                req_record = Some((idx, record));
                break;
            }
        }

        if let Some(_note) = record.note() {
            if _note.contains(&search) {
                req_record = Some((idx, record));
                break;
            }
        }
    }

    if let None = req_record {
        println!("[!] No Records were found with that phrase '{}'", search);
        log!("Password deletion no password's were found with the phrase '{}'", search);
        return;
    }
    
    let (idx, record) = req_record.unwrap();
    vault::record_fmt(vault::RecordPrint::RECORD(record.clone()));
    
    print!("[+] Do you want to delete this record ? (Y/n) : ");
    let choice = vault::fgets().to_lowercase();
    
    if !choice.is_empty() && choice.starts_with('n') {
        println!("[#] Record Wasnt Deleted!");
        return;
    }
    
    records.remove(idx);
    println!("[+] Record was Deleted!");

    vault::dump(&records, PASSWORDFILE, &password);
    
    log!("Record was Deleted");
}

fn import_credentials_from_json(path: String) {
    let password: String = rpassword::prompt_password("[+] Enter master password: ").unwrap();
    
    let mut records: Vec<vault::Record> = match vault::load(PASSWORDFILE, &password) {
        Ok(y) => match y {
            Some(x) => x,
            None => {
                println!("[!] No records were found!.\nTry 'rustsafe add' to create a new record");
                return;
            }
        },
        Err(err) => {
            if err.contains("[!] Error decrypting message") {
                println!("[!] Incorrect Password");
                logger::give_ban();
            }
            return;
        }
    };

    let foreign_passwd = rpassword::prompt_password("[+] Enter the password of the foreign json file: ").unwrap();

    let foreign_records: Vec<vault::Record> = match vault::load(PASSWORDFILE, &foreign_passwd) {
        Ok(y) => match y {
            Some(x) => x,
            None => {
                println!("[!] No records were found!.\nTry 'rustsafe add' to create a new record");
                return;
            }
        },
        Err(err) => {
            if err.contains("[!] Error decrypting message") {
                println!("[!] Incorrect Password"); // no need to ban for foreign records
            }
            return;
        }
    };

    for record in foreign_records {
        records.push(record);
    }

    vault::dump(&records, PASSWORDFILE, &password);
    println!("[+] Passwords were imported successfully from {}", path);

    log!("Passwords were imported successfully from {}", path);
}

fn export_credentials_to_json() {
    let password: String = rpassword::prompt_password("[+] Enter master password: ").unwrap();
    
    let records: Vec<vault::Record> = match vault::load(PASSWORDFILE, &password) {
        Ok(y) => match y {
            Some(x) => x,
            None => {
                println!("[!] No records were found!.\nTry 'rustsafe add' to create a new record");
                return;
            }
        },
        Err(err) => {
            if err.contains("[!] Error decrypting message") {
                println!("[!] Incorrect Password");
                logger::give_ban();
            }
            return;
        }
    };

    vault::dump(&records, EXPORTFILE, &password);
    println!("[+] Record was exported to '{}'", EXPORTFILE);

    log!("Record was exported to '{}'", EXPORTFILE);
}

