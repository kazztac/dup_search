use async_std::task;
use dup_search::args::OutputFormat;
use dup_search::hash::calculate_hashes_of;
use dup_search::hash::HashParam;
use dup_search::util::get_file_limit;
use dup_search::util::get_file_path_list_in;
use dup_search::Result;
use futures::channel::mpsc;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use log::*;
use multimap::MultiMap;

#[async_std::main]
async fn main() -> Result<()> {
    // Setup
    debug!("--- Start ---");
    pretty_env_logger::init_timed();

    // Parse program arguments
    debug!("Start: Arg setup");
    let total_start = std::time::Instant::now();
    let start = std::time::Instant::now();
    let args = dup_search::args::parse_args().unwrap();
    let hash_param = HashParam {
        algorithm: args.hash_algorithm(),
        buf_size: 1024 * 1024,
    };
    debug!("Finish: {:5}ms", start.elapsed().as_millis());

    // File search
    debug!("Start: File search");
    let start = std::time::Instant::now();
    let (mut tx, mut rx) = mpsc::unbounded();
    let cloned_folder_path = args.directory().to_string();
    task::spawn(async move {
        let _ = get_file_path_list_in(&cloned_folder_path, &mut tx).await;
    });
    let mut hash_senders = vec![];
    let mut hash_sender_index = 0usize;
    let ulimit_file_number = get_file_limit();
    while let Some(msg) = rx.next().await {
        let file_path = msg?;
        if hash_senders.len() <= ulimit_file_number {
            let (hash_tx, hash_rx) = mpsc::unbounded();
            let handle = task::spawn(calculate_hashes_of(hash_rx, hash_param.clone()));
            hash_senders.push((hash_tx, handle));
        }
        hash_senders[hash_sender_index].0.send(file_path).await?;
        hash_sender_index = (hash_sender_index + 1) % ulimit_file_number;
    }
    debug!("Finish: {:5}ms", start.elapsed().as_millis());

    // Calcurate hashes of files
    debug!("Start: Calcurating hashes");
    let start = std::time::Instant::now();
    let mut hash_files = MultiMap::new();
    for sender in hash_senders {
        sender.0.close_channel();
        let files = sender.1.await?;
        for (file_path, hash) in files {
            hash_files.insert(hash, file_path);
        }
    }
    debug!("Finish: {:5}ms", start.elapsed().as_millis());

    // Output the result
    debug!("Start Output");
    let start = std::time::Instant::now();
    let min: usize = args.filter_count_min();
    let max: usize = args.filter_count_max();
    let cloned_hash_files = hash_files.clone();
    for it in cloned_hash_files.iter_all() {
        let len = it.1.len();
        if len < min || len > max {
            hash_files.remove(it.0);
        }
    }
    let _output = match args.output_format() {
        OutputFormat::JSON => serde_json::to_string(&hash_files).unwrap(),
        OutputFormat::YAML => serde_yaml::to_string(&hash_files).unwrap(),
        OutputFormat::TOML => toml::to_string(&hash_files).unwrap(),
    };
    println!("{}", &_output);
    debug!("Finish: {:5}ms", start.elapsed().as_millis());
    debug!("--- Finish: {:5}ms ---", total_start.elapsed().as_millis());
    Ok(())
}
