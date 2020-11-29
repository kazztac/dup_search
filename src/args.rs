use crate::Result;
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use clap::{App, Arg};
use clap::{ArgMatches, Shell};
use std::ffi::OsString;

fn create_app() -> Result<App<'static, 'static>> {
    let app = app_from_crate!()
        .arg(Arg::from_usage("-g --hoge 'hoge hoge'"))
        .arg(Arg::from_usage("[directory] 'Target directory'").default_value("."));
    Ok(app)
}

pub fn parse_args() -> Result<ArgMatches<'static>> {
    Ok(create_app()?.get_matches())
}

pub fn generate_completion_file<S: Into<OsString>>(shell: Shell, out_dir: S) -> Result<()> {
    create_app()?.gen_completions(crate_name!(), shell, out_dir);
    Ok(())
}
