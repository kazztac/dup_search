use dup_search::hash::{calcurate_hashes_of, HashParam};
use dup_search::util::get_file_path_list_in;
use dup_search::Result;

#[async_std::main]
async fn main() -> Result<()> {
    let args = dup_search::args::parse_args().unwrap();
    let file_limit = String::from_utf8_lossy(
        &std::process::Command::new("ulimit")
            .arg("-n")
            .output()
            .unwrap()
            .stdout,
    )
    .trim()
    .parse()
    .unwrap_or(1024);
    println!("file_limit: {}, args: {:?}", file_limit, args);

    eprintln!("\n--- Start ---");
    let file_path_list = get_file_path_list_in(args.directory().to_string()).await?;
    let hash_files = calcurate_hashes_of(
        args.hash_algorithm(),
        file_path_list.iter().map(|s| &**s).collect(),
        HashParam {
            buf_size: 1024 * 1024,
            file_limit,
        },
    )
    .await?;
    for hash in hash_files {
        if hash.1.len() < 2 {
            continue;
        }
        //async_std::io::stdout().write(format!("{}: ", hash.0).as_bytes());
        println!("{}: ", hash.0);
        for file in hash.1 {
            //async_std::io::stdout().write(format!("              {}", file).as_bytes());
            println!("              {}", file);
        }
    }
    eprintln!("\n--- Finish ---");
    Ok(())

    //TODO: Replace output method to async-log crate.
    //TODO: Move logic to call file_limit to hash mod.
    //TODO: Control the number of files to open taking into ulimit setting.
    //TODO: Output result as a specified file format.
}
