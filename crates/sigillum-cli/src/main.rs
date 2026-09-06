//! Standalone command-line adapter for Sigillum.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use sigillum_core::ProductInfo;
use sigillum_openspec::Client;
use sigillum_store::{ApprovalStatus, ApprovalStore};

const HELP: &str = "Sigillum — contract-first orchestration for AI coding agents

Usage:
  sigillum [OPTIONS] [COMMAND]

Commands:
  approve    Record explicit approval for an exact contract fingerprint
  contract   Build a canonical snapshot from an OpenSpec change
  help       Print this help text
  version    Print version information

Contract options:
  --project <PATH>    OpenSpec project directory (default: current directory)
  --openspec <PATH>   OpenSpec executable (default: openspec on PATH)

Approval usage:
  sigillum approve <change> <fingerprint> [--project <path>] [--openspec <path>]

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
        Some("contract") => {
            let arguments = args.collect::<Vec<_>>();
            contract(&arguments)
        }
        Some("approve") => {
            let arguments = args.collect::<Vec<_>>();
            approve(&arguments)
        }
        Some(command) => {
            eprintln!("unknown command: {command}");
            eprintln!("run `sigillum help` for usage");
            ExitCode::from(2)
        }
    }
}

fn contract(arguments: &[OsString]) -> ExitCode {
    match parse_contract_arguments(arguments) {
        Ok(options) => run_contract(&options),
        Err(message) => {
            eprintln!("{message}");
            eprintln!("usage: sigillum contract <change> [--project <path>] [--openspec <path>]");
            ExitCode::from(2)
        }
    }
}

fn run_contract(options: &ContractOptions) -> ExitCode {
    let client = openspec_client(options);
    match client.load_contract(&options.project, &options.change) {
        Ok(loaded) => {
            let snapshot = loaded.snapshot();
            let approval_status = match ApprovalStore::open(loaded.planning_root())
                .and_then(|store| store.status(snapshot))
            {
                Ok(status) => status,
                Err(error) => {
                    eprintln!("contract failed: {error}");
                    return ExitCode::FAILURE;
                }
            };
            println!("OpenSpec {}", loaded.openspec_version());
            println!(
                "validation: valid ({} non-blocking issues)",
                loaded.validation_issue_count()
            );
            println!("change: {}", snapshot.change_id());
            println!("schema: {}", snapshot.openspec_schema());
            println!("contract: {}", snapshot.fingerprint());
            println!("artifacts: {}", snapshot.artifacts().len());
            for artifact in snapshot.artifacts() {
                println!(
                    "  {} {} {}",
                    artifact.artifact_id(),
                    artifact.relative_path(),
                    artifact.content_sha256()
                );
            }
            match approval_status {
                ApprovalStatus::Missing => {
                    println!("approval: missing");
                    ExitCode::SUCCESS
                }
                ApprovalStatus::Valid => {
                    println!("approval: valid");
                    ExitCode::SUCCESS
                }
                ApprovalStatus::Invalid(reason) => {
                    println!("approval: invalid ({reason})");
                    ExitCode::FAILURE
                }
            }
        }
        Err(error) => {
            eprintln!("contract failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn approve(arguments: &[OsString]) -> ExitCode {
    match parse_approve_arguments(arguments) {
        Ok(options) => run_approve(&options),
        Err(message) => {
            eprintln!("{message}");
            eprintln!(
                "usage: sigillum approve <change> <fingerprint> [--project <path>] [--openspec <path>]"
            );
            ExitCode::from(2)
        }
    }
}

fn run_approve(options: &ApproveOptions) -> ExitCode {
    let client = openspec_client(&options.contract);
    match client.load_contract(&options.contract.project, &options.contract.change) {
        Ok(loaded) => {
            let result = ApprovalStore::open(loaded.planning_root()).and_then(|store| {
                store.approve(loaded.snapshot(), &options.expected_fingerprint)
            });
            match result {
                Ok(path) => {
                    println!("approved: {}", loaded.snapshot().fingerprint());
                    println!("record: {}", path.display());
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("approval failed: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Err(error) => {
            eprintln!("approval failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn openspec_client(options: &ContractOptions) -> Client {
    options
        .openspec
        .as_ref()
        .map_or_else(Client::default, |path| Client::new(path.clone()))
}

#[derive(Debug, Eq, PartialEq)]
struct ContractOptions {
    change: String,
    project: PathBuf,
    openspec: Option<PathBuf>,
}

#[derive(Debug, Eq, PartialEq)]
struct ApproveOptions {
    contract: ContractOptions,
    expected_fingerprint: String,
}

fn parse_contract_arguments(arguments: &[OsString]) -> Result<ContractOptions, &'static str> {
    let change = arguments.first().ok_or("missing OpenSpec change name")?;
    parse_adapter_arguments(change, &arguments[1..])
}

fn parse_approve_arguments(arguments: &[OsString]) -> Result<ApproveOptions, &'static str> {
    let change = arguments.first().ok_or("missing OpenSpec change name")?;
    let expected_fingerprint = arguments
        .get(1)
        .ok_or("missing contract fingerprint")?
        .to_str()
        .ok_or("contract fingerprint must be valid UTF-8")?;
    if expected_fingerprint.len() != 64
        || !expected_fingerprint
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("contract fingerprint must be 64 lowercase hexadecimal characters");
    }
    let contract = parse_adapter_arguments(change, &arguments[2..])?;
    Ok(ApproveOptions {
        contract,
        expected_fingerprint: expected_fingerprint.to_owned(),
    })
}

fn parse_adapter_arguments(
    change: &OsString,
    arguments: &[OsString],
) -> Result<ContractOptions, &'static str> {
    let change = change
        .to_str()
        .ok_or("change name must be valid UTF-8")?;
    if change.starts_with('-') {
        return Err("the change name must be the first argument");
    }

    let mut project = std::env::current_dir().map_err(|_| "cannot read current directory")?;
    let mut openspec = None;
    let mut index = 0;
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

    use super::{parse_approve_arguments, parse_contract_arguments};

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

    #[test]
    fn approval_requires_exact_lowercase_fingerprint() {
        let valid = [
            OsString::from("add-auth"),
            OsString::from("a".repeat(64)),
        ];
        let invalid = [
            OsString::from("add-auth"),
            OsString::from("A".repeat(64)),
        ];

        assert!(parse_approve_arguments(&valid).is_ok());
        assert!(parse_approve_arguments(&invalid).is_err());
    }
}
