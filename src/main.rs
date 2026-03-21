/* Imports */
use std::{
    fs::{self, OpenOptions},
    io::{Result, Read}, env
};
use std::sync::OnceLock;
use rpassword;

/* Modules */
mod vault;
mod logger;
mod argparse;

/* Constants */
static PATH: OnceLock<String> = OnceLock::new();
static PASSWORDFILE: OnceLock<String> = OnceLock::new();
static EXPORTFILE: OnceLock<String> = OnceLock::new();
static LOG_FILE: OnceLock<String> = OnceLock::new();

type Commands = argparse::Commands;

struct DataDump {
    default: String,
    profiles: Vec<&'static[vault::Record]>
}

fn main() {
    if let None = set_paths() {
        println!("[!] Error: In setting paths!");
        return;
    }

    let (profile, command) = match argparse::parse_args(env::args()) {
        Some(x) => x,
        None => return
    };

    match command {
        Commands::Init => {
            if fs::exists(PATH.get().unwrap()).unwrap() {
                let val = format!("[+] DataBase Already Exists!");
                println!("{}", val);
                log!(ERROR, val);
                return;
            }

            match initialize_database() {
                Ok(()) => println!("[+] Database created successfully"),
                Err(x) => {
                    println!("[!] Error: {x}");
                    log!(ERROR, x.to_string());
                }
            }
        },

        Commands::Logs => print_logs(),

        Commands::Generate(size) => {
            let data: String = format!("Generated Password -> {}", vault::generate_rand_password(size));
            println!("{}", data);
            log!(INFO, data);
        },

        _ => {

            if !fs::exists(PATH.get().unwrap()).unwrap() {
                println!("[!] Database isn't created.\nTry '{} --init' to create a database",
                    env::args()
                    .nth(0)
                    .unwrap_or("".to_string())
                );
                return;
            }

            if let false = log!(LOG_FILE.get().unwrap()) {
                // user is banned probably
                return;
            }

            match command {
                Commands::Add(entry) => store_new_credential(entry),
                Commands::Get(entry) => display_stored_credentials(Some(entry)),
                Commands::List => display_stored_credentials(None),
                Commands::Edit(entry) => update_existing_credential(entry),
                Commands::Delete(entry) => remove_existing_credential(entry),
                Commands::Passwd => update_master_password(),
                Commands::Import(path) => import_credentials_from_json(path),
                Commands::Export => export_credentials_to_json(),

                /* Profile Setup */
                Commands::Default(profile) => set_default_profile(profile),
                Commands::CreateProfile(profile) => {},
                Commands::EditProfile(profile) => {},
                Commands::DeleteProfile(profile) => {},
                _ => {},
            }
        }
    }
}

fn set_default_profile(profile: String) {}

fn create_profile(profile: String) {}

fn edit_profile_name(profile: String) {}

fn delete_profile(profile: String) {}

fn set_paths() -> Option<()> {
    let dir_name = env::home_dir()?;
    PATH.set(format!("{}/.rustsafe", dir_name.display())).ok()?;
    PASSWORDFILE.set(format!("{}/.rustsafe/dump.json", dir_name.display())).ok()?;
    EXPORTFILE.set(format!("{}", dir_name.display())).ok()?;
    LOG_FILE.set(format!("{}/.rustsafe/log", dir_name.display())).ok()?;

    Some(())
}

fn print_logs() {
    if !fs::exists(LOG_FILE.get().unwrap()).unwrap() {
        let val = format!("[!] Error: Log file wasn't created!");
        println!("{}", val);
        log!(ERROR, val);
        return;
    }

    {
        let mut file = OpenOptions::new()
            .read(true)
            .open(LOG_FILE.get().unwrap())
            .unwrap();

        let mut buffer = String::new();
        let _ = file.read_to_string(&mut buffer).unwrap();
        println!("{}", buffer);
    }

    log!(INFO, "Logs were viewed");
}

fn initialize_database() -> Result<()> {
    fs::create_dir(PATH.get().unwrap())?;
    let _ = fs::File::create(PASSWORDFILE.get().unwrap())?;


    let _ = log!(LOG_FILE.get().unwrap());
    log!(INFO, "DataBase Created");
    Ok(())
}

fn display_stored_credentials(entry: Option<String>, profile: Option<String>) {
    let password: String = rpassword::prompt_password("[+] Enter master password: ").unwrap();

    let records: Vec<vault::Record> = match vault::load(PASSWORDFILE.get().unwrap(), &password, profile) {
        Ok(y) => match y {
            Some(x) => x,
            None => {
                println!("[!] No records were found!\nTry 'rustsafe --add' to create a new record");
                return;
            }
        },
        Err(err) => {
            if err.contains("[!] Error decrypting message") {
                println!("[!] Incorrect Password");
                log!(INVALID, "Incorrect Password");
            } else {
                println!("[!] Error: {err}");
            }
            return;
        }
    };

    if let None = entry {        // if list is called
        if records.is_empty() {
            println!("[!] No passwords were saved!\nTry 'rustsafe --add' to create a new record");
            log!(INFO, "All Records were viewed but database empty");
            return;
        }
        vault::record_fmt(vault::RecordPrint::VECTOR(records));
        log!(INFO, "All Records were viewed");
        return;
    }

    let search = entry.unwrap_or(String::from(""));
    let mut found = Vec::new();

    for record in records {
        if record.entry().contains(&search) || record.username().contains(&search) {
            found.push(record);
            continue;
        }

        if let Some(_email) = record.email() {
            if _email.contains(&search) {
                found.push(record);
                continue;
            }
        }

        if let Some(note) = record.note() {
            if note.contains(&search) {
                found.push(record);
                continue;
            }
        }
    }

    if found.is_empty() {
        println!("[!] Record with '{}' doesn't exists", search);
        return;
    }

    vault::record_fmt(vault::RecordPrint::VECTOR(found));
    log!(INFO, "Records were viewed");
}

fn store_new_credential(entry: String, profile: Option<String>) {
    let mut data: Vec<String> = vec![entry.clone()];

    let password: String = rpassword::prompt_password("[+] Enter master password: ").unwrap();

    let mut records: Vec<vault::Record> = match vault::load(PASSWORDFILE.get().unwrap(), &password, profile) {
        Ok(y) => match y {
            Some(x) => x,
            None => Vec::new(),
        },
        Err(err) => {
            if err.contains("[!] Error decrypting message") {
                println!("[!] Incorrect Password");
                log!(INVALID, "Incorrect Password");
            } else {
                println!("[!] Error: {err}");
            }
            return;
        }
    };

    print!("[+] Enter username for '{}': ", entry);
    data.push(vault::fgets());

    print!("[+] Enter password for '{}' (default length 30) : ", entry);
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

    vault::dump(&records, PASSWORDFILE.get().unwrap(), &password);

    println!("[+] Credentials was stored into the database!");
    log!(INFO, "New record was added to the database");
}

fn update_existing_credential(search: String, profile: Option<String>) {
    let password: String = rpassword::prompt_password("[+] Enter master password: ").unwrap();

    let mut records: Vec<vault::Record> = match vault::load(PASSWORDFILE.get().unwrap(), &password, profile) {
        Ok(y) => match y {
            Some(x) => x,
            None => {
                println!("[!] No records were found!\nTry 'rustsafe --add' to create a new record");
                return;
            }
        },
        Err(err) => {
            if err.contains("[!] Error decrypting message") {
                println!("[!] Incorrect Password");
                log!(INVALID, "Incorrect Password");
            } else {
                println!("[!] Error: {err}");
            }
            return;
        }
    };

    let mut req_record: Option<(usize, &vault::Record)> = None;

    for (idx, record) in records.iter().enumerate() {
        if record.username().contains(&search) || record.entry().contains(&search) {
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
        log!(INFO, format!("Password updation failed no password's were found with the phrase '{}'", search));
        return;
    }

    let (idx, record) = req_record.unwrap();
    vault::record_fmt(vault::RecordPrint::RECORD(record.clone()));

    print!("[+] Do you want to change this record ? (Y/n) : ");
    let choice = vault::fgets().to_lowercase();

    if choice.is_empty() || choice.starts_with('n') {
        println!("[#] Record Wasnt Updated!");
        return;
    }

    {
        let mut data: Vec<String> = Vec::new();
        print!("[+] Enter new username for '{}' (optional): ", (*record).entry());
        data.push((*record).entry());

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

        vault::dump(&records, PASSWORDFILE.get().unwrap(), &password);

        log!(INFO, format!("Credentials was updated with the phrase '{}'", search));
    }
}

fn update_master_password(profile: Option<String>) {
    let password: String = rpassword::prompt_password("[+] Enter master password: ").unwrap();

    let records: Vec<vault::Record> = match vault::load(PASSWORDFILE.get().unwrap(), &password, profile) {
        Ok(y) => match y {
            Some(x) => x,
            None => {
                println!("[!] No records were found!\nTry 'rustsafe --add' to create a new record");
                return;
            }
        },
        Err(err) => {
            if err.contains("[!] Error decrypting message") {
                println!("[!] Incorrect Password");
                log!(INVALID, "Incorrect Password");
            }
            return;
        }
    };

    let passwd: String = rpassword::prompt_password("[+] Enter new master password: ").unwrap();
    let _password: String = rpassword::prompt_password("[+] Enter new master password again: ").unwrap();

    if passwd != _password {
        println!("[!] Passwords doesn't match!");
        log!(INFO, "Master Password change failed, Passwords doesnt match");
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

    vault::dump(&new_records, PASSWORDFILE.get().unwrap(), &_password);

    println!("[+] Master password was changed successfully!");
    log!(INFO, "Master password was changed");
}

fn remove_existing_credential(search: String, profile: Option<String>) {
    let password: String = rpassword::prompt_password("[+] Enter master password: ").unwrap();

    let mut records: Vec<vault::Record> = match vault::load(PASSWORDFILE.get().unwrap(), &password, profile) {
        Ok(y) => match y {
            Some(x) => x,
            None => {
                println!("[!] No records were found!\nTry 'rustsafe --add' to create a new record");
                return;
            }
        },
        Err(err) => {
            if err.contains("[!] Error decrypting message") {
                println!("[!] Incorrect Password");
                log!(INVALID, "Incorrect Password");
            }
            return;
        }
    };

    let mut req_records: Vec<(usize, &vault::Record)> = Vec::new();

    for (idx, record) in records.iter().enumerate() {
        if record.username().contains(&search) || (*record).entry().contains(&search) {
            req_records.push((idx, record));
            continue;
        }

        if let Some(_email) = record.email() {
            if _email.contains(&search) {
                req_records.push((idx, record));
                continue;
            }
        }

        if let Some(_note) = record.note() {
            if _note.contains(&search) {
                req_records.push((idx, record));
                continue;
            }
        }
    }

    if req_records.len() == 0 {
        println!("[!] No Records were found with that phrase '{}'", search);
        log!(INFO, format!("Password deletion failed no password's were found with the phrase '{}'", search));
        return;
    }

    for (idx, record) in &req_records {
        vault::record_fmt(vault::RecordPrint::RECORD((*record).clone()));

        print!("[+] Do you want to delete this record ? (Y/n) : ");
        let choice = vault::fgets().to_lowercase();

        if choice.starts_with('y') {
            records.remove(*idx);
            println!("[+] Record was Deleted!");
            log!(INFO, "Record was Deleted");
            break;
        }

        println!("[#] Record Wasnt Deleted!");
    }

    vault::dump(&records, PASSWORDFILE.get().unwrap(), &password);
}

fn import_credentials_from_json(path: String, profile: Option<String>) {
    // Perfect it
    let password: String = rpassword::prompt_password("[+] Enter master password: ").unwrap();

    let mut records: Vec<vault::Record> = match vault::load(PASSWORDFILE.get().unwrap(), &password, profile) {
        Ok(y) => match y {
            Some(x) => x,
            None => {
                println!("[$] 0 passwords found in local database");
                Vec::new()
            }
        },
        Err(err) => {
            if err.contains("[!] Error decrypting message") {
                println!("[!] Incorrect Password");
                log!(INVALID, "Incorrect Password");
            }
            return;
        }
    };

    let foreign_passwd = rpassword::prompt_password(&format!("[+] Enter the password to {} file: ", path)).unwrap();

    let foreign_records: Vec<vault::Record> = match vault::load(&path, &foreign_passwd, profile) {
        Ok(y) => match y {
            Some(x) => x,
            None => {
                println!("[!] 0 records were found in the foreign database!\nTry 'rustsafe --add' to create a new record");
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

    vault::dump(&records, PASSWORDFILE.get().unwrap(), &password);
    println!("[+] Passwords were imported successfully from {}", path);

    log!(INFO, format!("Passwords were imported successfully from {}", path));
}

fn export_credentials_to_json() {
    // The whole file
    let password: String = rpassword::prompt_password("[+] Enter master password: ").unwrap();

    let records: Vec<vault::Record> = match vault::load(PASSWORDFILE.get().unwrap(), &password) {
        Ok(y) => match y {
            Some(x) => x,
            None => {
                println!("[!] No records were found to export!\nTry 'rustsafe --add' to create a new record");
                return;
            }
        },
        Err(err) => {
            if err.contains("[!] Error decrypting message") {
                println!("[!] Incorrect Password");
                log!(INVALID, "Incorrect Password");
            }
            return;
        }
    };

    vault::dump(&records, EXPORTFILE.get().unwrap(), &password);
    println!("[+] Record was exported to '{}'", EXPORTFILE.get().unwrap());

    log!(INFO, format!("Record was exported to '{}'", EXPORTFILE.get().unwrap()));
}
