#![allow(dead_code)]

use serde::{Serialize, Deserialize};
use std::io::{self, Write};
use std::fs;
use hex::{encode, decode};
use sha2::{Sha256, Digest};
use rand::random;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    salt: String,
    username: String,
    password: String,
    email: Option<String>,
    note: Option<String>,
    hmac: String,
}


impl Record {
    pub fn new(data: [String; 4], key: &str) -> Self {
        /* takes the fields in the arr and then joins the stuff which are not empty
         * HMAC calculation
         *     L-> concated all the non-empty fields and then add key to the last THEN hash256
         * */
        let bytes: Vec<u8> = (0..12).map(|_| { random::<u8>() }).collect();
        let mut calc_hash: String = data.iter().filter(|x| !x.is_empty()).collect()::<Vec<_>>().join("\n");
        calc_hash.push_str(key);

        Record {
            salt: encode(bytes),
            username: data[0].clone(),
            password: data[1].clone(),
            email: if data[2].is_empty() { None } else { Some(data[2].clone()) },
            note: if data[3].is_empty() { None } else { Some(data[3].clone()) },
            hmac: hash256(calc_hash),
        }
    }

    pub fn encrypt_record(&self, key: &str) -> Self {
        let nonce = decode(&self.salt).unwrap();
        let (mut email, mut note): (Option<String>, Option<String>) = (None, None);

        let username = match encrypt(key, &self.username, &nonce) {
            Ok(x) => x,
            Err(x) => panic!("{x}"),
        };

        let password = match encrypt(key, &self.password, &nonce) {
            Ok(x) => x,
            Err(x) => panic!("{x}"),
        };
        
        if let Some(_email) = &self.email {
            email = match encrypt(key, &_email, &nonce) {
                Ok(x) => Some(x),
                Err(x) => panic!("{x}"),
            };
        }
        
        if let Some(_note) = &self.note {
            note = match encrypt(key, &_note, &nonce) {
                Ok(x) => Some(x),
                Err(x) => panic!("{x}"),
            };
        }

        Record {
            salt: self.salt.clone(),
            username,
            password,
            email,
            note,
            hmac: self.hmac.clone(), // same hmac 
        }
    }

    pub fn decrypt_record(&self, key: &str) -> Self {
        let nonce = decode(&self.salt).unwrap();
        let mut calc_hash = String::new();
        let (mut email, mut note): (Option<String>, Option<String>) = (None, None);


        let username = match decrypt(key, &self.username, &nonce) {
            Ok(x) => { 
                calc_hash.push_str(&x);
                x
            },
            Err(x) => panic!("{x}"),
        };

        let password = match decrypt(key, &self.password, &nonce) {
            Ok(x) => { 
                calc_hash.push_str(&x);
                x
            },
            Err(x) => panic!("{x}"),
        };
        
        if let Some(_email) = &self.email {
            email = match decrypt(key, &_email, &nonce) {
                Ok(x) => { 
                    calc_hash.push_str(&x);
                    Some(x)
                },
                Err(x) => panic!("{x}"),
            };
        }
        
        if let Some(_note) = &self.note {
            note = match decrypt(key, &_note, &nonce) {
                Ok(x) => { 
                    calc_hash.push_str(&x);
                    Some(x)
                },           
                Err(x) => panic!("{x}"),
            };
        }
        
        calc_hash.push_str(key);
        
        /* 
         * Condition will be true work if password is changed 
         * That is when change_database_password() function in implemented 
         * this will panic so in change_database_password() function change all 
         * the hmac of each records
         *
         * */
        if hash256(calc_hash) != self.hmac {
            panic!("[!] Hashes doesnt match!");
        }

        Record {
            salt: self.salt.clone(),
            username,
            password,
            email,
            note,
            hmac: self.hmac.clone(),
        }
    }

    pub fn load(path: &str, key: &str) -> Option<Vec<Record>> {
        // TODO: hardcode the path to the passworfile 
        let data: String = match fs::read_to_string(path) {
            Ok(x) => x,
            Err(x) => panic!("[!] Error {x}"),
        };
        
        if data.len() == 0 {
            return None;
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
            decrypted_records.push(record.decrypt_record(&hash256(new_key)));
        }

        Some(decrypted_records)
    }

    pub fn dump(records: &[Record], path: &str, key: &str) {
        let mut encrypted_records: Vec<Record> = Vec::new();
        for record in records {
            // key = hash(nonce[..12] + key + nonce[12..])
            let mut new_key: String = String::new();
            new_key.push_str(&record.salt[..12]);
            new_key.push_str(key);
            new_key.push_str(&record.salt[12..]);
            encrypted_records.push((*record).encrypt_record(&hash256(new_key)));
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
}

pub fn fgets() -> String {
    let mut input = String::new();
    let _ = io::stdout().flush();
    io::stdin().read_line(&mut input).expect("[!] Error reading from stdin!");
    
    return input.trim().to_owned();
}

fn hash256(text: String) -> String {
    let res = Sha256::digest(text.as_bytes());
    return encode(res);
}

fn encrypt(key: &str, plaintext: &String, nonce: &[u8]) -> Result<String, String> {
    let key = Key::<Aes256Gcm>::from_slice(key.as_bytes());
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

fn decrypt(key: &str, ciphertext: &String, nonce: &[u8]) -> Result<String, String> {
    let key = Key::<Aes256Gcm>::from_slice(key.as_bytes());
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
            return Err(format!("[!] Error decrypting message: {}", x).to_owned());
        },
    };
 
    return Ok(String::from_utf8_lossy(&plaintext).to_string());
}
