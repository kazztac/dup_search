use crate::Result;
pub use clap::Shell;
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use clap::{App, Arg};
use std::ffi::OsString;

#[derive(Debug, Copy, Clone)]
pub enum HashAlgorithm {
    MD5,
    Blake3,
}

#[derive(Debug)]
pub struct ProgramArgs {
    hoge: bool,
    directory: String,
    algorithm: HashAlgorithm,
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
}

fn create_app() -> Result<App<'static, 'static>> {
    let app = app_from_crate!()
        .arg(Arg::from_usage("-g --hoge 'hoge hoge'"))
        .arg(
            Arg::from_usage("-a --algorithm 'Hash algorithm'")
                .possible_values(&["MD5", "Blake3"])
                .default_value("MD5"),
        )
        .arg(Arg::from_usage("[directory] 'Target directory'").default_value("."));
    Ok(app)
}

pub fn parse_args() -> Result<ProgramArgs> {
    let matches = create_app()?.get_matches();
    let algorithm = if matches.value_of("algorithm").unwrap() == "Blake3" {
        HashAlgorithm::Blake3
    } else {
        HashAlgorithm::MD5
    };
    Ok(ProgramArgs {
        hoge: matches.is_present("hoge"),
        directory: matches.value_of("directory").unwrap().to_string(),
        algorithm,
    })
}

pub fn generate_completion_file<S: Into<OsString>>(shell: Shell, out_dir: S) -> Result<()> {
    create_app()?.gen_completions(crate_name!(), shell, out_dir);
    Ok(())
}
