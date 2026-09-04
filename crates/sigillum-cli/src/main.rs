//! Standalone command-line adapter for Sigillum.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use sigillum_core::ProductInfo;
use sigillum_openspec::Client;

const HELP: &str = "Sigillum — contract-first orchestration for AI coding agents

Usage:
  sigillum [OPTIONS] [COMMAND]

Commands:
  contract   Build a canonical snapshot from an OpenSpec change
  help       Print this help text
  version    Print version information

Contract options:
  --project <PATH>    OpenSpec project directory (default: current directory)
  --openspec <PATH>   OpenSpec executable (default: openspec on PATH)

Options:
  -h, --help       Print help
  -V, --version    Print version information
";

fn main() -> ExitCode {
    let mut args = std::env::args_os();
    let _executable = args.next();

    match args.next().as_deref().and_then(|value| value.to_str()) {
        None | Some("help" | "-h" | "--help") => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("version" | "-V" | "--version") => {
            let product = ProductInfo::current();
            println!("{} {}", product.name, product.version);
            ExitCode::SUCCESS
        }
        Some("contract") => contract(args.collect()),
        Some(command) => {
            eprintln!("unknown command: {command}");
            eprintln!("run `sigillum help` for usage");
            ExitCode::from(2)
        }
    }
}

fn contract(arguments: Vec<OsString>) -> ExitCode {
    match parse_contract_arguments(&arguments) {
        Ok(options) => run_contract(&options),
        Err(message) => {
            eprintln!("{message}");
            eprintln!("usage: sigillum contract <change> [--project <path>] [--openspec <path>]");
            ExitCode::from(2)
        }
    }
}

fn run_contract(options: &ContractOptions) -> ExitCode {
    let client = options
        .openspec
        .as_ref()
        .map_or_else(Client::default, |path| Client::new(path.clone()));
    match client.load_contract(&options.project, &options.change) {
        Ok(loaded) => {
            let snapshot = loaded.snapshot();
            println!("OpenSpec {}", loaded.openspec_version());
            println!("change: {}", snapshot.change_id());
            println!("schema: {}", snapshot.openspec_schema());
            println!("contract: {}", snapshot.fingerprint());
            println!("artifacts: {}", snapshot.artifacts().len());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("contract failed: {error}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ContractOptions {
    change: String,
    project: PathBuf,
    openspec: Option<PathBuf>,
}

fn parse_contract_arguments(arguments: &[OsString]) -> Result<ContractOptions, &'static str> {
    let change = arguments
        .first()
        .ok_or("missing OpenSpec change name")?
        .to_str()
        .ok_or("change name must be valid UTF-8")?;
    if change.starts_with('-') {
        return Err("the change name must be the first argument");
    }

    let mut project = std::env::current_dir().map_err(|_| "cannot read current directory")?;
    let mut openspec = None;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index]
            .to_str()
            .ok_or("option names must be valid UTF-8")?;
        let value = arguments
            .get(index + 1)
            .ok_or("option requires a path value")?;
        match option {
            "--project" => project = PathBuf::from(value),
            "--openspec" => openspec = Some(PathBuf::from(value)),
            _ => return Err("unknown contract option"),
        }
        index += 2;
    }

    Ok(ContractOptions {
        change: change.to_owned(),
        project,
        openspec,
    })
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::PathBuf;

    use super::parse_contract_arguments;

    #[test]
    fn parses_contract_paths_as_os_strings() {
        let arguments = [
            OsString::from("add-auth"),
            OsString::from("--project"),
            OsString::from("project"),
            OsString::from("--openspec"),
            OsString::from("tools/openspec"),
        ];

        let options = parse_contract_arguments(&arguments).expect("valid arguments");

        assert_eq!(options.change, "add-auth");
        assert_eq!(options.project, PathBuf::from("project"));
        assert_eq!(options.openspec, Some(PathBuf::from("tools/openspec")));
    }

    #[test]
    fn requires_change_before_options() {
        let arguments = [OsString::from("--project"), OsString::from("project")];

        assert!(parse_contract_arguments(&arguments).is_err());
    }
}
