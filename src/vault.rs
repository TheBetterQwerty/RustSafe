use serde::{Serialize, Deserialize};
use std::io::{self, Write};
use hex::{encode, decode};
use sha2::{Sha512, Digest};
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
    email: String,
    note: String,
    hmac: String,
}

impl Record {
    pub fn new(data: [String; 4], key: &str) -> Self {
        let bytes: Vec<u8> = (0..12).map(|_| { random::<u8>() }).collect();
        let mut calc_hash: String = data.join("");
        calc_hash.push_str(key);

        Record {
            salt: encode(bytes),
            username: data[0].clone(),
            password: data[1].clone(),
            email: data[2].clone(),
            note: data[3].clone(),
            hmac: hash512(calc_hash),
        }
    }

    pub fn encrypt_record(&self, key: &str) -> Self {
        let nonce = decode(&self.salt).unwrap();
        
        let username = match encrypt(key, &self.username, &nonce) {
            Ok(x) => x,
            Err(x) => panic!("{x}"),
        };

        let password = match encrypt(key, &self.password, &nonce) {
            Ok(x) => x,
            Err(x) => panic!("{x}"),
        };

        let email = match encrypt(key, &self.email, &nonce) {
            Ok(x) => x,
            Err(x) => panic!("{x}"),
        };

        let note = match encrypt(key, &self.note, &nonce) {
            Ok(x) => x,
            Err(x) => panic!("{x}"),
        };

        Record {
            salt: self.salt.clone(),
            username,
            password,
            email,
            note,
            hmac: self.hmac.clone(),
        }
    }

    pub fn calculate_hmac(&mut self, data: String) {
        self.hmac.push_str(&hash512(data)); // data is all the fields decryted + key
    }

    pub fn decrypt_record(&self, key: &str) -> Self {
        let nonce = decode(&self.salt).unwrap();
        let mut calc_hash = String::new();

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

        let email = match decrypt(key, &self.email, &nonce) {
            Ok(x) => { 
                calc_hash.push_str(&x);
                x
            },
            Err(x) => panic!("{x}"),
        };

        let note = match decrypt(key, &self.note, &nonce) {
            Ok(x) => { 
                calc_hash.push_str(&x);
                x
            },           
            Err(x) => panic!("{x}"),
        };
        
        calc_hash.push_str(key);

        if hash512(calc_hash) != self.hmac {
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
}

pub fn fgets() -> String {
    let mut input = String::new();
    io::stdout().flush();
    io::stdin().read_line(&mut input).expect("[!] Error reading from stdin!");
    
    return input.trim().to_owned();
}

pub fn hash512(text: String) -> String {
    let res = Sha512::digest(text.as_bytes());
    return encode(res);
}

fn encrypt(key: &str, plaintext: &String, nonce: &[u8]) -> Result<String, String> {
    let key = Key::<Aes256Gcm>::from_slice(key.as_bytes());
    let nonce = Nonce::from_slice(nonce); // nonce must be a 12 byte shit 
    let cipher = Aes256Gcm::new(key);

    let ciphertext = match cipher.encrypt(nonce, (*plaintext).as_bytes()) {
        Ok(x) => x,
        Err(x) => {
            return Err(format!("[!] Error encrypting message: {}", x).to_owned());
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
            return Err(format!("[!] Error: {x}").to_string());
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
