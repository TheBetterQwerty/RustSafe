use std::{fs, io::{self, Write}, env};

mod vault;
mod logger;
mod argparse;

const PATH: &str = "/home/qwerty/.rustsafe";
const PASSWORDFILE: &str = "/home/qwerty/.rustsafe/dump.json";
const LOGFILE: &str = "/home/qwerty/.rustsafe/log";
const EXPORT: &str = "/home/qwerty/desktop/main.json";

type Commands = argparse::Commands;

fn main() {
    if !fs::exists(PATH).unwrap() {
        println!("[!] Database isn't created.\nTry 'rustsafe init' to create a database");
        return;
    }
    
    logger::start_logger(LOGFILE);

    match argparse::parse_args(env::args()) {
        Some(cmd) => {
            match cmd {
                Commands::Init => match initialize_database() {
                    Ok(()) => println!("[+] Database created successfully"),
                    Err(x) => println!("[!] Error: {x}")
                },
                Commands::Add(entry) => store_new_credential(entry),
                Commands::Generate(size) => println!("Generated Password -> {}", vault::generate_rand_password(size)),  
                Commands::Get(entry) => display_stored_credentials(Some(entry)),
                Commands::List => display_stored_credentials(None),
                Commands::Edit(entry) => update_existing_credential(entry),
                Commands::Delete(entry) => remove_existing_credential(entry),
                Commands::Passwd => update_master_password(),
                Commands::Import(path) => import_credentials_from_json(path),
                Commands::Export => export_credentials_to_json(),
                _ => {},
            }
        },
        None => {},
    };
}

fn initialize_database() -> io::Result<()> {
    fs::create_dir(PATH)?;
    let _ = fs::File::create(PASSWORDFILE)?;
    let _ = fs::File::create(LOGFILE)?;
    log!("Database created successfully");

    Ok(())
}

fn display_stored_credentials(entry: Option<String>) {
    print!("[+] Enter master password: ");
    let password: String = vault::fgets();
    
    let records: Vec<vault::Record> = match vault::load(PASSWORDFILE, &password) {
        Some(x) => x,
        None => {
            println!("[!] No records were found!.\nTry 'rustsafe add' to create a new record");
            return;
        }
    };
    
    let search: String;
    let mut found: bool = false;

    if let Some(x) = entry {
        search = x;
    } else {
        print!("[+] Enter username or email or entry name to search: ");
        search = vault::fgets();
    }
    
    for record in records {
        if record.entry() == search || record.username() == search {
            println!("Record Found: {:?}", record);
            found = true;
            break;
        }
        
        if let Some(_email) = record.email() {
            if _email == search {
                println!("Record Found {:?}", record);
                found = true;
                break;
            }
        }

        if let Some(note) = record.note() {
            if note == search {
                println!("Record Found: {:?}", record);
                found = true;
                break;
            }
        }
    }

    if !found {
        println!("[!] Record with '{}' doesn't exists", search);
    }

    log!("Records were viewed");
}

fn store_new_credential(entry: String) {
    let mut data: Vec<String> = vec![entry.clone()];

    print!("[+] Enter master password: ");
    let password: String = vault::fgets();
    
    let mut records = match vault::load(PASSWORDFILE, &password) {
        Some(x) => x,
        None => {
            println!("[!] No records were found!.\nTry 'rustsafe add' to create a new record");
            return;
        },
    };

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

    records.push(vault::Record::new(&data, &password));
    
    vault::dump(&records, PASSWORDFILE, &password);

    log!("New record was added to the database");
}

fn update_existing_credential(search: String) {
    print!("[+] Enter master password: ");
    let password: String = vault::fgets();

    let mut records: Vec<vault::Record> = match vault::load(PASSWORDFILE, &password) {
        Some(x) => x,
        None => {
            println!("[!] No records were found!.\nTry 'rustsafe add' to create a new record");
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
            if _email == search {
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
    record.pretty_print();
    
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
    print!("[+] Enter master password: ");
    let password: String = vault::fgets();

    let records: Vec<vault::Record> = match vault::load(PASSWORDFILE, &password) {
        Some(x) => x,
        None => {
            println!("[!] No records were found!.\nTry 'rustsafe add' to create a new record");
            return;
        }
    };
    
    print!("[+] Enter new master password: ");
    let passwd: String = vault::fgets();
    print!("[+] Enter new master password: ");
    let _password: String = vault::fgets();
    
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
    log!("Master password was changed");
}

fn remove_existing_credential(search: String) {
    print!("[+] Enter master password: ");
    let password: String = vault::fgets();

    let mut records: Vec<vault::Record> = match vault::load(PASSWORDFILE, &password) {
        Some(x) => x,
        None => {
            println!("[!] No records were found!.\nTry 'rustsafe add' to create a new record");
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
            if _email == search {
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
    record.pretty_print();
    
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
    print!("[+] Enter master password: ");
    let password: String = vault::fgets();

    let mut records: Vec<vault::Record> = match vault::load(PASSWORDFILE, &password) {
        Some(x) => x,
        None => {
            println!("[!] No records were found!.\nTry 'rustsafe add' to create a new record");
            return;
        }
    };

    print!("[+] Enter the password of the foreign json file: ");
    let foreign_passwd = vault::fgets();

    let foreign_records: Vec<vault::Record> = match vault::load(&path, &foreign_passwd) {
        Some(x) => x,
        None => {
            println!("Error in Something");
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
    // just copy the file to the EXPORTFILE
}

