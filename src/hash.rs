use crate::args::HashAlgorithm;
use crate::Result;
use async_std::fs::File;
use async_std::io::BufReader;
use async_std::{prelude::*, task};
use digest::Digest;
use futures::channel::mpsc;
use futures::SinkExt;
use hex::ToHex;
use multimap::MultiMap;

#[derive(Debug, Clone)]
pub struct HashParam {
    pub algorithm: HashAlgorithm,
    pub buf_size: usize,
}

impl Default for HashParam {
    fn default() -> Self {
        Self {
            algorithm: HashAlgorithm::Blake3,
            buf_size: 1024 * 1024,
        }
    }
}

fn get_file_limit() -> usize {
    String::from_utf8_lossy(
        &std::process::Command::new("ulimit")
            .arg("-n")
            .output()
            .unwrap()
            .stdout,
    )
    .trim()
    .parse::<usize>()
    .map_or_else(
        |e| {
            println!(
                "As couldn't get file limit value, use default value. Error: {}",
                e
            );
            1024usize
        },
        |v| v,
    )
}

async fn calculate_hash_of<D: Digest>(file_path: &str, param: &HashParam) -> Result<String> {
    let file = File::open(file_path).await?;
    let mut buf_read = BufReader::new(file);
    let mut buf = Vec::with_capacity(param.buf_size);
    unsafe {
        buf.set_len(param.buf_size);
    }
    let mut digest = D::new();
    loop {
        let read_size = buf_read.read(&mut buf).await?;
        if read_size <= 0 {
            break;
        }
        digest.update(buf[..read_size].as_ref());
    }

    Ok(digest.finalize().as_slice().encode_hex())
}

async fn calculate_hashes_of_internal(
    file_path_list: Vec<String>,
    param: HashParam,
) -> Vec<(String, String)> {
    let mut result = Vec::with_capacity(file_path_list.len());
    for file_path in file_path_list {
        let hash = match &param.algorithm {
            &HashAlgorithm::MD5 => calculate_hash_of::<md5::Md5>(&file_path, &param).await,
            &HashAlgorithm::Blake3 => calculate_hash_of::<blake3::Hasher>(&file_path, &param).await,
        }
        .unwrap();
        result.push((file_path, hash));
    }
    result
}

pub async fn calcurate_hashes_of(
    file_path_list: &Vec<String>,
    param: &HashParam,
    progress_sender: Option<mpsc::UnboundedSender<Result<usize>>>,
) -> Result<MultiMap<String, String>> {
    let file_limit = get_file_limit();
    let file_count_at_once = (file_path_list.len() / file_limit) + 1;
    let mut handles = vec![];
    for chunked_file_path_list in file_path_list.chunks(file_count_at_once) {
        let cloned_chunked_file_path_list = chunked_file_path_list
            .iter()
            .map(|it| it.clone())
            .collect::<Vec<String>>();
        let cloned_param = param.clone();
        let handle = task::spawn(calculate_hashes_of_internal(
            cloned_chunked_file_path_list,
            cloned_param,
        ));
        handles.push(handle);
    }
    let mut hash_and_file_path_map = MultiMap::new();
    for handle in handles {
        let result = handle.await;
        if let Some(sender) = &progress_sender {
            let mut sender = sender.clone();
            sender.send(Ok(result.len())).await?;
        }
        for (file_path, hash) in result {
            hash_and_file_path_map.insert(hash, file_path);
        }
    }
    Ok(hash_and_file_path_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::get_file_path_list_in;
    use apply_method::*;
    use lazy_static::*;
    use std::collections::HashMap;

    lazy_static! {
        pub static ref EXACT_FILES: HashMap<&'static str, HashMap<HashAlgorithm, &'static str>> = {
            HashMap::new()
                .apply(|it| {
                    it.insert(
                        "./resource/test/test1.png",
                        HashMap::new().apply(|it| {
                            it.insert(HashAlgorithm::MD5, "01e3a3cdac7ae3023aff5c2c51a6345d");
                            it.insert(
                                HashAlgorithm::Blake3,
                                "1bc859358071abc29058e30efb6853a0ec0b04093d9c04537f1e63dd0ccbe9af",
                            );
                        }),
                    )
                })
                .apply(|it| {
                    it.insert(
                        "./resource/test/dirA/test2.png",
                        HashMap::new().apply(|it| {
                            it.insert(HashAlgorithm::MD5, "bbf39ea418ff93373f7fe25e2d889ebc");
                            it.insert(
                                HashAlgorithm::Blake3,
                                "7ea88a79e87b46677cabfb93200c7a32e85ba9a40c3bb56bf5d943396b4a9c0c",
                            );
                        }),
                    )
                })
                .apply(|it| {
                    it.insert(
                        "./resource/test/dirA/test1_copy.png",
                        HashMap::new().apply(|it| {
                            it.insert(HashAlgorithm::MD5, "01e3a3cdac7ae3023aff5c2c51a6345d");
                            it.insert(
                                HashAlgorithm::Blake3,
                                "1bc859358071abc29058e30efb6853a0ec0b04093d9c04537f1e63dd0ccbe9af",
                            );
                        }),
                    )
                })
                .apply(|it| {
                    it.insert(
                        "./resource/test/dirA/dirA2/test3.png",
                        HashMap::new().apply(|it| {
                            it.insert(HashAlgorithm::MD5, "25d1e8a77689bcf68556ccc8b13c1a66");
                            it.insert(
                                HashAlgorithm::Blake3,
                                "e5a0b01e8179d7e0368186303489c06caaac551430a7eee0e1e2c472380e0931",
                            );
                        }),
                    )
                })
                .apply(|it| {
                    it.insert(
                        "./resource/test/dirB/a.txt",
                        HashMap::new().apply(|it| {
                            it.insert(HashAlgorithm::MD5, "60b725f10c9c85c70d97880dfe8191b3");
                            it.insert(
                                HashAlgorithm::Blake3,
                                "81c4b7f7e0549f1514e9cae97cf40cf133920418d3dc71bedbf60ec9bd6148cb",
                            );
                        }),
                    )
                })
        };
    }

    fn generate_exact_hashes(algorithm: HashAlgorithm) -> Vec<&'static str> {
        EXACT_FILES
            .iter()
            .map(|it| *it.1.get(&algorithm).unwrap())
            .collect::<Vec<&str>>()
            .apply(|it| {
                it.sort();
                it.dedup();
            })
    }

    fn generate_exact_files(
        algorithm: HashAlgorithm,
        exact_hash: &'static str,
    ) -> Vec<&'static str> {
        EXACT_FILES
            .iter()
            .filter(|it| *it.1.get(&algorithm).unwrap() == exact_hash)
            .map(|it| *it.0)
            .collect::<Vec<&str>>()
            .apply(|it| it.sort())
    }

    #[test]
    fn test_calculate_hash_of_file_path_with_md5() {
        task::block_on(async {
            let file_path = "./resource/test/test1.png";
            let exact_hash = EXACT_FILES.get(file_path).unwrap();
            let hash = calculate_hash_of::<md5::Md5>(file_path, &Default::default())
                .await
                .unwrap();
            assert_eq!(&hash, exact_hash.get(&HashAlgorithm::MD5).unwrap());
        });
    }

    #[test]
    fn test_calculate_hash_of_file_path_with_blake3() {
        task::block_on(async {
            let file_path = "./resource/test/test1.png";
            let exact_hash = EXACT_FILES.get(file_path).unwrap();
            let hash = calculate_hash_of::<blake3::Hasher>(file_path, &Default::default())
                .await
                .unwrap();
            assert_eq!(&hash, exact_hash.get(&HashAlgorithm::Blake3).unwrap());
        });
    }

    #[test]
    fn test_calculate_hashes_of_internal_md5() {
        task::block_on(async {
            let files = get_file_path_list_in("./resource/test".to_string())
                .await
                .unwrap();
            let param = HashParam::default().apply(|it| it.algorithm = HashAlgorithm::MD5);
            let results = calculate_hashes_of_internal(files, param).await;
            assert_eq!(results.len(), EXACT_FILES.len());
            for exact_item in EXACT_FILES.iter() {
                let exact_file = exact_item.0;
                let exact_hash = exact_item.1.get(&HashAlgorithm::MD5).unwrap();
                let (result_file, result_hash) =
                    results.iter().find(|it| &it.0 == *exact_file).unwrap();
                assert_eq!(result_file, exact_file);
                assert_eq!(result_hash, exact_hash);
            }
        });
    }

    #[test]
    fn test_calculate_hashes_of_internal_blake3() {
        task::block_on(async {
            let files = get_file_path_list_in("./resource/test".to_string())
                .await
                .unwrap();
            let param = HashParam::default().apply(|it| it.algorithm = HashAlgorithm::Blake3);
            let results = calculate_hashes_of_internal(files, param).await;
            assert_eq!(results.len(), EXACT_FILES.len());
            for exact_item in EXACT_FILES.iter() {
                let exact_file = exact_item.0;
                let exact_hash = exact_item.1.get(&HashAlgorithm::Blake3).unwrap();
                let (result_file, result_hash) =
                    results.iter().find(|it| &it.0 == *exact_file).unwrap();
                assert_eq!(result_file, exact_file);
                assert_eq!(result_hash, exact_hash);
            }
        });
    }

    #[test]
    fn test_culcurate_hashes_of_files() {
        task::block_on(async {
            let exact_hashes = generate_exact_hashes(HashAlgorithm::Blake3);
            let files = get_file_path_list_in("./resource/test".to_string())
                .await
                .unwrap();
            let param = HashParam::default();
            let hash_files = calcurate_hashes_of(&files, &param, None).await.unwrap();
            assert_eq!(hash_files.len(), exact_hashes.len());
            for exact_hash in exact_hashes {
                let exact_files = generate_exact_files(HashAlgorithm::Blake3, exact_hash);
                let files = hash_files
                    .get_vec(exact_hash)
                    .unwrap()
                    .clone()
                    .apply(|it| it.sort());
                assert_eq!(files, exact_files);
            }
        });
    }

    #[test]
    fn test_culcurate_hashes_of_files_with_progress_check() {
        task::block_on(async {
            let exact_hashes = generate_exact_hashes(HashAlgorithm::Blake3);
            let files = get_file_path_list_in("./resource/test".to_string())
                .await
                .unwrap();
            let param = HashParam::default();
            let (tx, mut rx) = mpsc::unbounded();
            let handle =
                task::spawn(async move { calcurate_hashes_of(&files, &param, Some(tx)).await });
            let mut progress_count = 0;
            while let Some(event) = rx.next().await {
                let recv_count = event.unwrap();
                println!("Recv: {}", recv_count);
                progress_count += recv_count;
            }
            assert_eq!(progress_count, EXACT_FILES.len());
            let hash_files = handle.await.unwrap();
            assert_eq!(hash_files.len(), exact_hashes.len());
            for exact_hash in exact_hashes {
                let exact_files = generate_exact_files(HashAlgorithm::Blake3, exact_hash);
                let files = hash_files
                    .get_vec(exact_hash)
                    .unwrap()
                    .clone()
                    .apply(|it| it.sort());
                assert_eq!(files, exact_files);
            }
        });
    }

    #[test]
    #[ignore]
    fn test_get_file_limit() {
        let file_limit = get_file_limit();
        println!("file_limit: {}", file_limit);
    }
}
