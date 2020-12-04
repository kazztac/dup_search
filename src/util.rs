use crate::Result;
use async_recursion::async_recursion;
use async_std::fs::read_dir;
use async_std::prelude::*;

#[async_recursion]
pub async fn get_file_path_list_in(folder_path: String) -> Result<Vec<String>> {
    let mut result = vec![];
    let mut entries = read_dir(folder_path).await?;
    while let Some(res) = entries.next().await {
        let entry = res?;
        let path = entry.path();
        if path.is_dir().await {
            let path = path.to_str().unwrap().to_string();
            let mut files = get_file_path_list_in(path).await?;
            result.append(&mut files);
        } else {
            result.push(path.as_path().to_str().unwrap().to_string());
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::*;
    use async_std::task;

    lazy_static! {
        pub static ref EXACT_FILES: Vec<&'static str> = vec![
            "./resource/test/test1.png",
            "./resource/test/dirA/test2.png",
            "./resource/test/dirA/test1_copy.png",
            "./resource/test/dirA/dirA2/test3.png",
            "./resource/test/dirB/a.txt",
        ];
    }

    #[test]
    fn test_get_file_path_list_in_folder() {
        task::block_on(async {
            let file_path_list = get_file_path_list_in("./resource/test".to_string())
                .await
                .unwrap();
            assert_eq!(file_path_list.len(), EXACT_FILES.len());
            for exact_file_path in EXACT_FILES.iter() {
                assert!(file_path_list
                    .iter()
                    .find(|it| it == exact_file_path)
                    .is_some());
            }
        });
    }
}
