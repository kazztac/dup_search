use crate::Result;
pub use clap::Shell;
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use clap::{App, Arg};
use std::ffi::OsString;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum HashAlgorithm {
    MD5,
    Blake3,
}

#[derive(Debug, Copy, Clone)]
pub enum OutputFormat {
    JSON,
    TOML,
    YAML,
}

#[derive(Debug)]
pub struct ProgramArgs {
    hoge: bool,
    directory: String,
    algorithm: HashAlgorithm,
    output_format: OutputFormat,
}

impl ProgramArgs {
    pub fn is_hoge(&self) -> bool {
        self.hoge
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
}

fn create_app() -> Result<App<'static, 'static>> {
    let app = app_from_crate!()
        .arg(Arg::from_usage("-g --hoge 'hoge hoge'"))
        .arg(
            Arg::from_usage("-a --algorithm 'Hash algorithm'")
                .possible_values(&["MD5", "Blake3"])
                .default_value("Blake3"),
        )
        .arg(
            Arg::from_usage("-f --format 'Output Format'")
                .possible_values(&["json", "toml", "yaml"])
                .default_value("json"),
        )
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
        "yaml" => OutputFormat::YAML,
        "toml" => OutputFormat::TOML,
        _ => OutputFormat::JSON,
    };
    Ok(ProgramArgs {
        hoge: matches.is_present("hoge"),
        directory: matches.value_of("directory").unwrap().to_string(),
        algorithm,
        output_format,
    })
}

pub fn generate_completion_file<S: Into<OsString>>(shell: Shell, out_dir: S) -> Result<()> {
    create_app()?.gen_completions(crate_name!(), shell, out_dir);
    Ok(())
}
