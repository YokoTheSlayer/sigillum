//! Standalone command-line adapter for Sigillum.

#![forbid(unsafe_code)]

use std::process::ExitCode;

use sigillum_core::ProductInfo;

const HELP: &str = "Sigillum — contract-first orchestration for AI coding agents

Usage:
  sigillum [OPTIONS] [COMMAND]

Commands:
  help       Print this help text
  version    Print version information

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
        Some(command) => {
            eprintln!("unknown command: {command}");
            eprintln!("run `sigillum help` for usage");
            ExitCode::from(2)
        }
    }
}
