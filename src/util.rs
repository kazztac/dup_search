use crate::Result;
use async_recursion::async_recursion;
use async_std::fs::read_dir;
use async_std::prelude::*;
use async_std::task;
use futures::channel::mpsc;
use futures::SinkExt;

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

#[async_recursion]
pub async fn get_file_path_list_in2(
    folder_path: &str,
    sender: &mut mpsc::UnboundedSender<Result<String>>,
) -> Result<()> {
    let mut entries = read_dir(folder_path).await?;
    while let Some(res) = entries.next().await {
        let entry = res?;
        let path = entry.path();
        let file_path = path.to_str().unwrap().to_string();
        if path.is_dir().await {
            let mut cloned_sender = sender.clone();
            task::spawn(async move {
                if let Err(e) = get_file_path_list_in2(&file_path, &mut cloned_sender).await {
                    let _ignore = cloned_sender.send(Err(e)).await;
                }
            });
        } else {
            sender.send(Ok(file_path)).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task;
    use futures::stream::StreamExt;
    use lazy_static::*;

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
    #[test]
    fn test_get_file_path_list_in_folder2() {
        task::block_on(async {
            let (tx, mut rx) = mpsc::unbounded();
            task::spawn(async move {
                let _ignore = get_file_path_list_in2("./resource/test", &mut tx.clone()).await;
            });
            let mut file_path_list = vec![];
            while let Some(msg) = rx.next().await {
                let file_path = msg.unwrap();
                file_path_list.push(file_path);
            }
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
