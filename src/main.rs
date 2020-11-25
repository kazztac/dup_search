use async_recursion::async_recursion;
use async_std::fs::{read_dir, File};
use async_std::io::BufReader;
use async_std::{prelude::*, task};
use digest::Digest;
use hex::ToHex;
use multimap::MultiMap;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn calculate_hash_of<D: Digest>(file_path: &str) -> Result<String> {
    let file = File::open(file_path).await?;
    let mut buf_read = BufReader::new(file);
    let mut buf = [0u8; 1024];
    let mut digest = D::new();
    loop {
        let read_size = buf_read.read(&mut buf).await?;
        if read_size <= 0 {
            break;
        }
        digest.update(buf[0..read_size].as_ref());
    }

    Ok(digest.finalize().as_slice().encode_hex())
}

async fn calcurate_hashes_of(file_path_list: Vec<&str>) -> Result<MultiMap<String, &str>> {
    let mut handles = vec![];
    for file_path in file_path_list {
        let cloned_file_path = file_path.to_string();
        let handle =
            task::spawn(async move { calculate_hash_of::<md5::Md5>(&cloned_file_path).await });
        //task::spawn(async move { calculate_hash_of::<blake3::Hasher>(&cloned_file_path).await });
        handles.push((handle, file_path));
    }
    let mut hash_and_file_path_map = MultiMap::new();
    for (handle, file_path) in handles {
        let hash = handle.await?;
        hash_and_file_path_map.insert(hash, file_path);
    }
    Ok(hash_and_file_path_map)
}

#[async_recursion]
async fn get_file_path_list_in(folder_path: String) -> Result<Vec<String>> {
    let mut result = vec![];
    let mut entries = read_dir(folder_path).await?;
    while let Some(res) = entries.next().await {
        let entry = res?;
        let path = entry.path();
        if path.is_dir().await {
            let path = path.to_str().unwrap().to_string();
            let mut files = task::spawn(get_file_path_list_in(path)).await?;
            result.append(&mut files);
        } else {
            result.push(path.as_path().to_str().unwrap().to_string());
        }
    }
    Ok(result)
}

async fn run() -> Result<()> {
    let file_path_list = get_file_path_list_in(".".to_string()).await?;
    let hash_files = calcurate_hashes_of(file_path_list.iter().map(|s| &**s).collect()).await?;
    for hash in hash_files {
        if hash.1.len() < 2 {
            continue;
        }
        println!("{}: ", hash.0);
        for file in hash.1 {
            println!("              {}", file);
        }
    }
    Ok(())
}

fn main() {
    eprintln!("--- Start ---");
    task::block_on(async { run().await.unwrap() });
    eprintln!("--- Finish ---");

    //TODO: Read args from command line.
    //TODO: Open a file specified in args.
    //TODO: Search files recursively from a folder specified in args.
    //TODO: Output result as a specified file format.
}

#[cfg(test)]
mod tests {
    use super::*;
    use apply_method::*;
    use lazy_static::*;
    use std::collections::HashMap;

    lazy_static! {
        pub static ref EXACT_FILES: HashMap<&'static str, &'static str> = {
            HashMap::new()
                .apply(|it| {
                    it.insert(
                        "./resource/test/test1.png",
                        "01e3a3cdac7ae3023aff5c2c51a6345d",
                    )
                })
                .apply(|it| {
                    it.insert(
                        "./resource/test/dirA/test2.png",
                        "bbf39ea418ff93373f7fe25e2d889ebc",
                    )
                })
                .apply(|it| {
                    it.insert(
                        "./resource/test/dirA/test1_copy.png",
                        "01e3a3cdac7ae3023aff5c2c51a6345d",
                    )
                })
                .apply(|it| {
                    it.insert(
                        "./resource/test/dirA/dirA2/test3.png",
                        "25d1e8a77689bcf68556ccc8b13c1a66",
                    )
                })
                .apply(|it| {
                    it.insert(
                        "./resource/test/dirB/a.txt",
                        "60b725f10c9c85c70d97880dfe8191b3",
                    )
                })
        };
    }

    #[test]
    fn test_calculate_hash_of_file_path() {
        task::block_on(async {
            let file_path = "./resource/test/test1.png";
            let exact_hash = EXACT_FILES.get(file_path).unwrap();
            let hash = calculate_hash_of::<md5::Md5>(file_path).await.unwrap();
            assert_eq!(&hash, exact_hash);
        });
    }

    #[test]
    fn test_get_file_path_list_in_folder() {
        task::block_on(async {
            let file_path_list = get_file_path_list_in("./resource/test".to_string())
                .await
                .unwrap();
            let exact_file_path_list = EXACT_FILES.keys();
            assert_eq!(file_path_list.len(), exact_file_path_list.len());
            for exact_file_path in exact_file_path_list {
                assert!(file_path_list
                    .iter()
                    .find(|it| it == exact_file_path)
                    .is_some());
            }
        });
    }

    #[test]
    fn test_culcurate_hashes_of_files() {
        task::block_on(async {
            let exact_hashes = EXACT_FILES
                .iter()
                .map(|it| *it.1)
                .collect::<Vec<&str>>()
                .apply(|it| {
                    it.sort();
                    it.dedup();
                });
            let files = get_file_path_list_in("./resource/test".to_string())
                .await
                .unwrap();
            let hash_files = calcurate_hashes_of(files.iter().map(|s| &**s).collect())
                .await
                .unwrap();
            assert_eq!(hash_files.len(), exact_hashes.len());
            for exact_hash in exact_hashes {
                let exact_files = EXACT_FILES
                    .iter()
                    .filter(|it| *it.1 == exact_hash)
                    .map(|it| *it.0)
                    .collect::<Vec<&str>>()
                    .apply(|it| it.sort());
                let files = hash_files
                    .get_vec(exact_hash)
                    .unwrap()
                    .clone()
                    .apply(|it| it.sort());
                assert_eq!(files, exact_files);
            }
        });
    }
}
