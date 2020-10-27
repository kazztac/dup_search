use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

//fn sha256(b: &Vec<u8>) -> String {
//    use sha2::Digest;
//    format!("{:x}", sha2::Sha256::digest(b))
//}

fn md5(b: &Vec<u8>) -> String {
    format!("{:x}", md5::compute(b))
}

fn main() {
    let file_path = "./resource/test/hoge.jpg";
    let file = File::open(file_path).unwrap();
    let mut buf_read = BufReader::new(file);
    let mut buf = Vec::new();
    let read_length = buf_read.read_to_end(&mut buf).unwrap();
    println!("read_length: {}", read_length);
    //println!("{:?}", buf);

    let algorithm = md5;
    //let algorithm = sha256;
    let hashed_value = algorithm(&buf);
    println!("hashed_value: {:?}", hashed_value);
   
    //TODO: Search files recursively from a current folder.
    //TODO: Hold hash values for searched files.
    //TODO: Output all duplicated files.
    //TODO: Use async for all file-io.
    //TODO: Read args from command line.
    //TODO: Open a file specified in args.
    //TODO: Search files recursively from a folder specified in args.
    //TODO: Output result as a specified file format.
}
