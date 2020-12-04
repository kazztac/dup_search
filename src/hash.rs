use crate::args::HashAlgorithm;
use crate::Result;
use async_std::fs::File;
use async_std::io::BufReader;
use async_std::{prelude::*, task};
use digest::Digest;
use hex::ToHex;
use multimap::MultiMap;

#[derive(Debug, Clone)]
pub struct HashParam {
    pub buf_size: usize,
    pub file_limit: usize,
}

impl Default for HashParam {
    fn default() -> Self {
        Self {
            buf_size: 1024 * 1024,
            file_limit: 1024,
        }
    }
}

async fn calculate_hash_of<D: Digest>(file_path: &str, param: HashParam) -> Result<String> {
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

pub async fn calcurate_hashes_of(
    algorithm: HashAlgorithm,
    file_path_list: Vec<&str>,
    param: HashParam,
) -> Result<MultiMap<String, &str>> {
    let mut handles = vec![];
    for file_path in file_path_list {
        let cloned_file_path = file_path.to_string();
        let cloned_param = param.clone();
        let handle = task::spawn(async move {
            match algorithm {
                HashAlgorithm::MD5 => {
                    calculate_hash_of::<md5::Md5>(&cloned_file_path, cloned_param).await
                }
                HashAlgorithm::Blake3 => {
                    calculate_hash_of::<blake3::Hasher>(&cloned_file_path, cloned_param).await
                }
            }
        });
        handles.push((handle, file_path));
    }
    let mut hash_and_file_path_map = MultiMap::new();
    let sum = handles.len();
    for (i, entry) in handles.into_iter().enumerate() {
        let (handle, file_path) = entry;
        let hash = handle.await?;
        hash_and_file_path_map.insert(hash, file_path);
        //async_std::io::stdout().write(format!("\r{:5} / {:5}", i + 1, sum).as_bytes());
        print!("\r{:5} / {:5}", i + 1, sum);
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

    #[test]
    fn test_calculate_hash_of_file_path_with_md5() {
        task::block_on(async {
            let file_path = "./resource/test/test1.png";
            let exact_hash = EXACT_FILES.get(file_path).unwrap();
            let hash = calculate_hash_of::<md5::Md5>(file_path, Default::default())
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
            let hash = calculate_hash_of::<blake3::Hasher>(file_path, Default::default())
                .await
                .unwrap();
            assert_eq!(&hash, exact_hash.get(&HashAlgorithm::Blake3).unwrap());
        });
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
    fn test_culcurate_hashes_of_files_with_md5() {
        task::block_on(async {
            let exact_hashes = generate_exact_hashes(HashAlgorithm::MD5);
            let files = get_file_path_list_in("./resource/test".to_string())
                .await
                .unwrap();
            let hash_files = calcurate_hashes_of(
                HashAlgorithm::MD5,
                files.iter().map(|s| &**s).collect(),
                Default::default(),
            )
            .await
            .unwrap();
            assert_eq!(hash_files.len(), exact_hashes.len());
            for exact_hash in exact_hashes {
                let exact_files = generate_exact_files(HashAlgorithm::MD5, exact_hash);
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
    fn test_culcurate_hashes_of_files_with_blake3() {
        task::block_on(async {
            let exact_hashes = generate_exact_hashes(HashAlgorithm::Blake3);
            let files = get_file_path_list_in("./resource/test".to_string())
                .await
                .unwrap();
            let hash_files = calcurate_hashes_of(
                HashAlgorithm::Blake3,
                files.iter().map(|s| &**s).collect(),
                Default::default(),
            )
            .await
            .unwrap();
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
}
