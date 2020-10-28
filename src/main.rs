use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::read_dir;

#[allow(dead_code)]
fn sha256(b: &Vec<u8>) -> String {
    use sha2::Digest;
    format!("{:x}", sha2::Sha256::digest(b))
}

fn md5(b: &Vec<u8>) -> String {
    format!("{:x}", md5::compute(b))
}

fn calcurate_hash_by(file_path: &str) -> String {
    let file = File::open(file_path).unwrap();
    let mut buf_read = BufReader::new(file);
    let mut buf = Vec::new();
    let read_length = buf_read.read_to_end(&mut buf).unwrap();
    println!("read_length: {}", read_length);
    //println!("{:?}", buf);

    let algorithm = md5;
    //let algorithm = sha256;
    let hashed_value = algorithm(&buf);
    hashed_value
}

fn get_file_list_in(folder_path: &str) -> Vec<String> {
    let mut result = vec![];
    let entries = read_dir(folder_path).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            let mut files = get_file_list_in(&path.as_path().to_str().unwrap());
            result.append(&mut files);
        }
        else {
            result.push(path.as_path().to_str().unwrap().to_string());
        }
    }
    result
}

fn main() {

    let files = get_file_list_in(".");
    println!("files: {:?}", files);

    let file_path = "./resource/test/hoge.jpg";
    let hashed_value = calcurate_hash_by(file_path);
    println!("hashed_value: {:?}", hashed_value);
   
    //TODO: Hold hash values for searched files.
    //TODO: Output all duplicated files.
    //TODO: Use async for all file-io.
    //TODO: Read args from command line.
    //TODO: Open a file specified in args.
    //TODO: Search files recursively from a folder specified in args.
    //TODO: Output result as a specified file format.
}
