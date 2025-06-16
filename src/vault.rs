use serde::{Serialize, Deserialize};
use std::io::{self, Write};
use std::fs;
use hex::{encode, decode};
use sha2::{Sha256, Digest};
use rand::{rng, random, distr::{Alphanumeric, SampleString}};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    salt: String,
    entry: String,
    username: String,
    password: String,
    email: Option<String>,
    note: Option<String>,
    hmac: String,
}


impl Record {
    pub fn new(data: &[String], key: &str) -> Self {
        /* 
         * HMAC calculation:
         *  o salt + (fields that are not empty) + key
         *  o sha256 hash 
         *
         * */
        let bytes: Vec<u8> = (0..12).map(|_| { random::<u8>() }).collect();
        let salt = encode(bytes);

        let mut vec = data
            .iter()
            .filter(|field| !field.is_empty())
            .cloned()                                    // &Vec<String> -> Vec<String>
            .collect::<Vec<String>>();
        
        vec.insert(0, salt.clone());
        vec.push(key.to_string());
        
        let calc_hash = vec.join("\n");

        Record {
            salt,
            entry: data[0].clone(),
            username: data[1].clone(),
            password: data[2].clone(),
            email: if data[3].is_empty() { None } else { Some(data[3].clone()) },
            note: if data[4].is_empty() { None } else { Some(data[4].clone()) },
            hmac: encode(hash256(calc_hash)),
        }
    }

    fn encrypt_record(&self, key: &[u8]) -> Self {
        let nonce = decode(&self.salt).unwrap();
        let (mut email, mut note) = (None, None);

        let username = match encrypt(key, &self.username, &nonce) {
            Ok(x) => x,
            Err(x) => panic!("[!] Error encrypting: {x}"),
        };

        let password = match encrypt(key, &self.password, &nonce) {
            Ok(x) => x,
            Err(x) => panic!("[!] Error encrypting: {x}"),
        };
        
        if let Some(_email) = &self.email {
            email = match encrypt(key, &_email, &nonce) {
                Ok(x) => Some(x),
                Err(x) => panic!("[!] Error encrypting: {x}"),
            };
        }
        
        if let Some(_note) = &self.note {
            note = match encrypt(key, &_note, &nonce) {
                Ok(x) => Some(x),
                Err(x) => panic!("[!] Error encrypting: {x}"),
            };
        }

        Record {
            salt: self.salt.clone(),
            entry: self.entry.clone(),
            username,
            password,
            email,
            note,
            hmac: self.hmac.clone(), // same hmac 
        }
    }

    fn decrypt_record(&self, key: &[u8], key_plain: &str) -> Result<Self,String> {
        let nonce = decode(&self.salt).unwrap();
        let mut calc_hash = String::from(self.entry.clone());
        let (mut email, mut note) = (None, None);

        let username = match decrypt(key, &self.username, &nonce) {
            Ok(x) => { 
                calc_hash.push_str(&x);
                x
            },
            Err(x) => return Err(x),
        };
       
        let password = match decrypt(key, &self.password, &nonce) {
            Ok(x) => { 
                calc_hash.push_str(&x);
                x
            },
            Err(x) => return Err(x),
        };

        if let Some(_email) = &self.email {
            email = match decrypt(key, &_email, &nonce) {
                Ok(x) => { 
                    calc_hash.push_str(&x);
                    Some(x)
                },
                Err(x) => return Err(x),
            };
        }
        
        if let Some(_note) = &self.note {
            note = match decrypt(key, &_note, &nonce) {
                Ok(x) => { 
                    calc_hash.push_str(&x);
                    Some(x)
                },           
                Err(x) => return Err(x),
            };
        }

        calc_hash.push_str(key_plain);
        
        if encode(hash256(calc_hash)) == self.hmac {
            panic!("[!] Hashes doesnt match! Tamparing Detected");
        }

        Ok(Record {
            salt: self.salt.clone(),
            entry: self.entry.clone(),
            username,
            password,
            email,
            note,
            hmac: self.hmac.clone(),
        })
    }
    
    pub fn pretty_print(&self) {
        let mut lines: Vec<String> = vec![
            format!("o Username: {}", self.username),
            format!("o Password: {}", self.password)
        ];
            
        if let Some(email) = &self.email {
            lines.insert(1, format!("o Email: {}", email));
        }

        if let Some(note) = &self.note {
            lines.push(format!("o Note: {}", note));
        }

        let width: usize = lines.iter().map(|x| (*x).len()).max().unwrap_or(0);
        
        println!("|{:_^width$}|", self.entry, width = width);

        for line in lines {
            println!("{line}");
        }
    }

    // entry username email note
    pub fn entry(&self) -> String {
        self.entry.clone()
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub fn password(&self) -> String {
        self.password.clone()
    }

    pub fn email(&self) -> Option<String> {
        self.email.clone()
    }

    pub fn note(&self) -> Option<String> {
        self.note.clone()
    }
}

pub fn load(path: &str, key: &str) -> Result<Option<Vec<Record>>, String> {
    // TODO: send the result too to log and give ban
    let data: String = match fs::read_to_string(path) {
        Ok(x) => x,
        Err(x) => panic!("[!] Error {x}"),
    };

    if data.len() == 0 {
        return Ok(None);
    }

    let records: Vec<Record> = match serde_json::from_str(&data) {
        Ok(x) => x,
        Err(x) => panic!("[!] Error {x}"),
    };

    /* Decrypt the records */
    let mut decrypted_records: Vec<Record> = Vec::new();
    for record in records {
        // key = hash(nonce[..12] + key + nonce[12..])
        let mut new_key: String = String::new();
        new_key.push_str(&record.salt[..12]);
        new_key.push_str(key);
        new_key.push_str(&record.salt[12..]);
        let key_bytes = hash256(new_key);
        let decrypted_data = match record.decrypt_record(&key_bytes, &key) {
            Ok(x) => x,
            Err(x) => return Err(x),
        };
        decrypted_records.push(decrypted_data);
    }

    Ok(Some(decrypted_records))
}

pub fn dump(records: &[Record], path: &str, key: &str) {
    let mut encrypted_records: Vec<Record> = Vec::new();

    for record in records {
        // key = hash(nonce[..12] + key + nonce[12..])
        let mut new_key: String = String::new();
        new_key.push_str(&record.salt[..12]);
        new_key.push_str(key);
        new_key.push_str(&record.salt[12..]);
        let key_bytes = hash256(new_key);
        encrypted_records.push((*record).encrypt_record(&key_bytes));
    }

    let encoded = match serde_json::to_string_pretty(&encrypted_records) {
        Ok(x) => x,
        Err(x) => panic!("[!] Error {x}"),
    };

    match fs::write(path, encoded.as_bytes()) {
        Ok(()) => {},
        Err(x) => panic!("[!] Error {x}"),
    }
}

pub fn generate_rand_password(size: usize) -> String {
    Alphanumeric.sample_string(&mut rng(), size)
}

pub fn fgets() -> String {
    let mut input = String::new();
    let _ = io::stdout().flush();
    io::stdin().read_line(&mut input).expect("[!] Error reading from stdin!");
    
    return input.trim().to_owned();
}

fn hash256(text: String) -> [u8; 32] { 
    let res = Sha256::digest(text.as_bytes());
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&res);
    bytes
}

fn encrypt(key: &[u8], plaintext: &String, nonce: &[u8]) -> Result<String, String> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let nonce = Nonce::from_slice(nonce); // nonce must be a 12 byte shit 
    let cipher = Aes256Gcm::new(key);

    let ciphertext = match cipher.encrypt(nonce, (*plaintext).as_bytes()) {
        Ok(x) => x,
        Err(x) => {
            return Err(format!("[!] Error encrypting message: {}", x));
        },
    };

    return Ok(encode(ciphertext));
}

fn decrypt(key: &[u8], ciphertext: &String, nonce: &[u8]) -> Result<String, String> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let nonce = Nonce::from_slice(nonce); // nonce must be a 12 byte shit 
    let cipher = Aes256Gcm::new(key);
    let ciphertext = match decode(ciphertext) {
        Ok(x) => x,
        Err(x) => {
            return Err(format!("[!] Error: {x}"));
        },
    };

    let plaintext = match cipher.decrypt(nonce, ciphertext.as_ref()) {
        Ok(x) => x,
        Err(x) => {
            return Err(format!("[!] Error decrypting message: {}", x));
        },
    };
 
    return Ok(String::from_utf8_lossy(&plaintext).to_string());
}
