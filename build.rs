use std::io::Error;
use std::process::ExitCode;

use npm_rs::{NodeEnv, NpmEnv};

fn main() -> Result<ExitCode, Error> {
    let exit_status = NpmEnv::default()
        .with_node_env(&NodeEnv::from_cargo_profile().unwrap_or_default())
        .init_env()
        .install(None)
        .run("build")
        .exec()?;
    if exit_status.success() {
        Ok(ExitCode::SUCCESS)
    } else {
        match exit_status.code() {
            Some(code) => eprintln!("npm exited with code {code}"),
            None => eprintln!("npm interrupted by signal"),
        }
        Ok(ExitCode::FAILURE)
    }
}
