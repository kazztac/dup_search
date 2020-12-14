use async_std::task;
use dup_search::args::OutputFormat;
use dup_search::hash::{calcurate_hashes_of, HashParam};
use dup_search::util::get_file_path_list_in;
use dup_search::Result;
use futures::channel::mpsc;
use futures::stream::StreamExt;
use log::info;
use mpsc::UnboundedReceiver;

async fn print_progress(mut rx: UnboundedReceiver<usize>, total: usize) {
    let mut progress_count = 0;
    while let Some(recv_count) = rx.next().await {
        progress_count += recv_count;
        print!("\r{:5} / {:5}", progress_count, total);
    }
    println!();
}

#[async_std::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();
    info!("--- Start ---");
    let total_start = std::time::Instant::now();
    info!("Start: Arg setup");
    let start = std::time::Instant::now();
    let args = dup_search::args::parse_args().unwrap();
    let hash_param = HashParam {
        algorithm: args.hash_algorithm(),
        buf_size: 1024 * 1024,
    };
    info!("Finish: {:5}ms", start.elapsed().as_millis());
    info!("Start: File search");
    let start = std::time::Instant::now();
    let (mut tx, mut rx) = mpsc::unbounded();
    let cloned_folder_path = args.directory().to_string();
    task::spawn(async move {
        let _ignore = get_file_path_list_in(&cloned_folder_path, &mut tx).await;
    });
    let mut file_path_list = vec![];
    while let Some(msg) = rx.next().await {
        let file_path = msg?;
        file_path_list.push(file_path);
    }
    info!("Finish: {:5}ms", start.elapsed().as_millis());
    info!("Start: Calcurating hashes");
    let start = std::time::Instant::now();
    let progress_sender = if args.is_verbose() {
        let (tx, rx) = mpsc::unbounded();
        task::spawn(print_progress(rx, file_path_list.len()));
        Some(tx)
    } else {
        None
    };
    let hash_files = calcurate_hashes_of(&file_path_list, &hash_param, progress_sender)
        .await
        .unwrap();
    info!("Finish: {:5}ms", start.elapsed().as_millis());
    info!("Start Output");
    let start = std::time::Instant::now();
    let _output = match args.output_format() {
        OutputFormat::JSON => serde_json::to_string(&hash_files).unwrap(),
        OutputFormat::YAML => serde_yaml::to_string(&hash_files).unwrap(),
        OutputFormat::TOML => toml::to_string(&hash_files).unwrap(),
    };
    println!("{}", &_output);
    info!("Finish: {:5}ms", start.elapsed().as_millis());
    info!("--- Finish: {:5}ms ---", total_start.elapsed().as_millis());
    Ok(())

    //TODO: Performance optimization.
    //  done: 1. The first task will search the file paths and send it by a channel.
    //  done 2. If the task find a directory, it spawns a new task searches there.
    //  3. If the channel notifies new message, it spawns a new task that calcurates the hash of
    //     the file until the number of the task will be the ulimit size.
    //  4. The task spawned on #3 retrieves the file path from the storage that be shared each
    //     tasks.
    //  5. The task notifies the result of calcuration by using a channel.
    //TODO: Filter the target files by specifing as an arg.
    //TODO: Filter the output files depends on args specified as min count and max count.
}
