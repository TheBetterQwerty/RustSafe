use std::fs;

mod vault;

const PATH: &str = "/home/qwerty/.rustsafe";
const PASSWORDFILE: &str = "/home/qwerty/.rustsafe";
const EXPORT: &str = "/home/qwerty/.rustsafe";

fn main() {
    if !fs::exists(PATH).unwrap() {
        
    }
}
