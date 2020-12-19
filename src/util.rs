use crate::Result;
use async_recursion::async_recursion;
use async_std::fs::read_dir;
use async_std::prelude::*;
use async_std::task;
use futures::channel::mpsc;
use futures::SinkExt;

#[async_recursion]
pub async fn get_file_path_list_in(
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
                if let Err(e) = get_file_path_list_in(&file_path, &mut cloned_sender).await {
                    let _ignore = cloned_sender.send(Err(e)).await;
                }
            });
        } else {
            sender.send(Ok(file_path)).await?;
        }
    }
    Ok(())
}

pub fn get_file_limit() -> usize {
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
            let (tx, mut rx) = mpsc::unbounded();
            task::spawn(async move {
                let _ignore = get_file_path_list_in("./resource/test", &mut tx.clone()).await;
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

    #[test]
    #[ignore]
    fn test_get_file_limit() {
        let file_limit = get_file_limit();
        println!("file_limit: {}", file_limit);
    }
}
