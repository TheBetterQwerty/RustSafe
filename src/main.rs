use std::fs;

mod vault;
mod logger;

const PATH: &str = "/home/qwerty/.rustsafe";
const PASSWORDFILE: &str = "/home/qwerty/.rustsafe";
const EXPORT: &str = "/home/qwerty/.rustsafe";

fn main() {
    if !fs::exists(PATH).unwrap() {
        
    }
}
