use crate::Result;
pub use clap::Shell;
use clap::{app_from_crate, arg_enum, crate_authors, crate_description, crate_name, crate_version};
use clap::{App, Arg};
use std::ffi::OsString;

arg_enum! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    pub enum HashAlgorithm {
        MD5,
        Blake3,
    }
}

arg_enum! {
    #[derive(Debug, Copy, Clone)]
    pub enum OutputFormat {
        JSON,
        TOML,
        YAML,
    }
}

#[derive(Debug)]
pub struct ProgramArgs {
    verbose: bool,
    directory: String,
    algorithm: HashAlgorithm,
    output_format: OutputFormat,
    filter_count_min: usize,
    filter_count_max: usize,
}

impl ProgramArgs {
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }
    pub fn directory(&self) -> &str {
        self.directory.as_str()
    }
    pub fn hash_algorithm(&self) -> HashAlgorithm {
        self.algorithm
    }
    pub fn output_format(&self) -> OutputFormat {
        self.output_format
    }
    pub fn filter_count_min(&self) -> usize {
        self.filter_count_min
    }
    pub fn filter_count_max(&self) -> usize {
        self.filter_count_max
    }
}

fn create_app() -> Result<App<'static, 'static>> {
    let app = app_from_crate!()
        .arg(Arg::from_usage("-v --verbose 'Verbose'"))
        .arg(
            Arg::from_usage("-a --algorithm 'Hash algorithm'")
                .possible_values(&HashAlgorithm::variants())
                .default_value("Blake3"),
        )
        .arg(
            Arg::from_usage("-f --format 'Output Format'")
                .possible_values(&OutputFormat::variants())
                .default_value("JSON"),
        )
        .arg(
            Arg::from_usage("--min 'Minimum duplicate count'")
                .min_values(0)
                .default_value("2"),
        )
        .arg(Arg::from_usage("--max 'Maxinum duplicate count'").max_values(std::usize::MAX as u64))
        .arg(Arg::from_usage("[directory] 'Target directory'").default_value("."));
    Ok(app)
}

pub fn parse_args() -> Result<ProgramArgs> {
    let matches = create_app()?.get_matches();
    let algorithm = match matches.value_of("algorithm").unwrap() {
        "MD5" => HashAlgorithm::MD5,
        _ => HashAlgorithm::Blake3,
    };
    let output_format = match matches.value_of("format").unwrap() {
        "YAML" | "yaml" => OutputFormat::YAML,
        "TOML" | "toml" => OutputFormat::TOML,
        _ => OutputFormat::JSON,
    };
    let filter_count_min = matches.value_of("min").unwrap().parse().unwrap();
    let filter_count_max = match matches.value_of("max") {
        Some(max) => {
            let max = max.parse().unwrap();
            if filter_count_min > max {
                filter_count_min
            } else {
                max
            }
        }
        None => std::usize::MAX,
    };
    Ok(ProgramArgs {
        verbose: matches.is_present("verbose"),
        directory: matches.value_of("directory").unwrap().to_string(),
        algorithm,
        output_format,
        filter_count_min,
        filter_count_max,
    })
}

pub fn generate_completion_file<S: Into<OsString>>(shell: Shell, out_dir: S) -> Result<()> {
    create_app()?.gen_completions(crate_name!(), shell, out_dir);
    Ok(())
}
