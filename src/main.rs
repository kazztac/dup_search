use dup_search::async_println;
use dup_search::hash::{calcurate_hashes_of, HashParam};
use dup_search::util::get_file_path_list_in;
use dup_search::Result;

#[async_std::main]
async fn main() -> Result<()> {
    let args = dup_search::args::parse_args().unwrap();
    let hash_param = HashParam {
        algorithm: args.hash_algorithm(),
        buf_size: 1024 * 1024,
    };
    async_println!("\n--- Start ---").await;
    let file_path_list = get_file_path_list_in(args.directory().to_string()).await?;
    let hash_files = calcurate_hashes_of(
        file_path_list.iter().map(|s| &**s).collect(),
        hash_param,
    )
    .await?;
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

    //TODO: Control the number of files to open taking into ulimit setting.
    //TODO: Use channel to nofity the results of each tasks.
    //TODO: Output result as a specified file format.
}
