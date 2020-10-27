use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    let file = File::open("./resource/test/hoge.jpg").unwrap();
    let mut buf_read = BufReader::new(file);
    let mut buf = Vec::new();
    let read_length = buf_read.read_to_end(&mut buf).unwrap();
    println!("read_length: {}", read_length);
    //println!("{:?}", buf);

    let md5 = md5::compute(buf);
    println!("md5: {:?}", md5);
   
    //TODO: Read args from command line.
    //TODO: Open a file specified in args.
    //TODO: Calculate a hash value by original algorithm.
    //TODO: Search files recursively from a folder specified in args.
    //TODO: Hold hash values for searched files.
    //TODO: Output all duplicated files.
    //TODO: Output result as a specified file format.
    //TODO: Use async for all file-io.
}
