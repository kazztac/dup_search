use crate::args::HashAlgorithm;
use crate::Result;
use async_std::fs::File;
use async_std::io::BufReader;
use async_std::prelude::*;
use digest::Digest;
use futures::channel::mpsc;
use hex::ToHex;

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

pub async fn calculate_hashes_of(
    mut rx: mpsc::UnboundedReceiver<String>,
    param: HashParam,
) -> Result<Vec<(String, String)>> {
    let mut result = vec![];
    while let Some(file_path) = rx.next().await {
        let hash = match &param.algorithm {
            &HashAlgorithm::MD5 => calculate_hash_of::<md5::Md5>(&file_path, &param).await,
            &HashAlgorithm::Blake3 => calculate_hash_of::<blake3::Hasher>(&file_path, &param).await,
        }?;
        result.push((file_path, hash));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::get_file_path_list_in;
    use apply_method::*;
    use async_std::task;
    use futures::sink::SinkExt;
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
    fn test_calculate_hashes_of() {
        task::block_on(async {
            let param = HashParam::default().apply(|it| it.algorithm = HashAlgorithm::MD5);
            let (tx_file, mut rx_file) = mpsc::unbounded();
            task::spawn(async move {
                let _ignore = get_file_path_list_in("./resource/test", &mut tx_file.clone()).await;
            });
            let (mut tx_hash, rx_hash) = mpsc::unbounded();
            task::spawn(async move {
                while let Some(file_path) = rx_file.next().await {
                    let _ignore = tx_hash.send(file_path.unwrap()).await;
                }
            });
            let result = calculate_hashes_of(rx_hash, param).await.unwrap();
            assert_eq!(result.len(), EXACT_FILES.len());
            for exact_item in EXACT_FILES.iter() {
                let exact_file = exact_item.0;
                let exact_hash = exact_item.1.get(&HashAlgorithm::MD5).unwrap();
                let (result_file, result_hash) =
                    result.iter().find(|it| &it.0 == *exact_file).unwrap();
                assert_eq!(result_file, exact_file);
                assert_eq!(result_hash, exact_hash);
            }
        });
    }
}
