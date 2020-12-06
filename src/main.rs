use async_std::task;
use dup_search::async_print;
use dup_search::async_println;
use dup_search::hash::{calcurate_hashes_of, HashParam};
use dup_search::util::get_file_path_list_in;
use dup_search::Result;
use futures::channel::mpsc;
use futures::stream::StreamExt;

#[async_std::main]
async fn main() -> Result<()> {
    let args = dup_search::args::parse_args().unwrap();
    let hash_param = HashParam {
        algorithm: args.hash_algorithm(),
        buf_size: 1024 * 1024,
    };
    async_println!("\n--- Start ---").await;
    let file_path_list = get_file_path_list_in(args.directory().to_string()).await?;
    let total = file_path_list.len();
    let (tx, mut rx) = mpsc::unbounded();
    let handle =
        task::spawn(
            async move { calcurate_hashes_of(&file_path_list, &hash_param, Some(tx)).await },
        );
    let mut progress_count = 0;
    while let Some(event) = rx.next().await {
        progress_count += event.unwrap();
        async_print!("\r{:5} / {:5}", progress_count, total).await;
    }
    async_println!().await;

    let hash_files = handle.await.unwrap();
    for hash in hash_files {
        if hash.1.len() < 2 {
            continue;
        }
        async_println!("{}: ", hash.0).await;
        for file in hash.1 {
            async_println!("              {}", file).await;
        }
    }
    async_println!("\n--- Finish ---").await;
    Ok(())

    //TODO: Output result as a specified file format.
}
