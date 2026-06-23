mod cli;
mod contract;
mod lint;
mod report;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();
    let result = run(&cli);
    match result {
        Ok(()) => {}
        Err(e) => {
            let code = e.exit_code();
            if cli.is_json() {
                let err_json = serde_json::json!({
                    "ok": false,
                    "error": {
                        "code": e.error_code(),
                        "message": e.to_string(),
                    }
                });
                eprintln!("{}", serde_json::to_string_pretty(&err_json).unwrap_or_else(|_| format!("{{\"ok\":false,\"error\":{{\"message\":\"{e}\"}}}}")));
            } else {
                eprintln!("error: {e}");
            }
            std::process::exit(code);
        }
    }
}

fn run(cli: &Cli) -> Result<(), KeystoneError> {
    let repo = cli.resolve_repo()?;

    match &cli.command {
        Command::Init => {
            if contract::contract_exists(&repo) {
                println!("Contract already exists at .agent-contract.toml");
                return Ok(());
            }
            let default = contract::Contract::default();
            contract::save(&repo, &default)?;
            println!("Created .agent-contract.toml with default rules");
            Ok(())
        }
        Command::Lint => {
            let contract = contract::load(&repo)?
                .ok_or_else(|| KeystoneError::Validation(
                    "No .agent-contract.toml found. Run `keystone init` first.".into()
                ))?;
            let result = lint::lint(&repo, &contract);
            report::print_lint(&result, cli.is_json())
        }
        Command::Show => {
            let contract = contract::load(&repo)?
                .ok_or_else(|| KeystoneError::Validation(
                    "No .agent-contract.toml found. Run `keystone init` first.".into()
                ))?;
            report::print_contract(&contract, cli.is_json())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeystoneError {
    #[error("{0}")]
    Validation(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl KeystoneError {
    pub fn exit_code(&self) -> i32 {
        match self {
            KeystoneError::Validation(_) => 1,
            KeystoneError::Io(_) => 2,
            KeystoneError::Json(_) => 1,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            KeystoneError::Validation(_) => "validation_error",
            KeystoneError::Io(_) => "io_error",
            KeystoneError::Json(_) => "json_error",
        }
    }
}
