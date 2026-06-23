use crate::contract::Contract;
use crate::lint::{LintResult, LintSeverity};
use crate::KeystoneError;

pub fn print_lint(result: &LintResult, is_json: bool) -> Result<(), KeystoneError> {
    if is_json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "lint": {
                "pass": result.pass,
                "findings": result.findings,
            }
        }))?);
    } else {
        if result.findings.is_empty() {
            println!("keystone lint: all checks passed ✓");
        } else {
            println!("keystone lint: {} finding(s)", result.findings.len());
            println!();
            for f in &result.findings {
                let icon = match f.severity {
                    LintSeverity::Error => "✗",
                    LintSeverity::Warning => "⚠",
                    LintSeverity::Info => "·",
                };
                println!("  {icon} [{}] {}", f.rule, f.message);
            }
            println!();
            if result.pass {
                println!("  Verdict: pass (warnings only)");
            } else {
                println!("  Verdict: FAIL");
            }
        }
    }
    Ok(())
}

pub fn print_contract(contract: &Contract, is_json: bool) -> Result<(), KeystoneError> {
    if is_json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "contract": contract,
        }))?);
    } else {
        println!("keystone contract:");
        println!();

        if !contract.project.required_files.is_empty() {
            println!("  Required files:");
            for f in &contract.project.required_files {
                println!("    - {f}");
            }
        }

        if !contract.safe_zones.is_empty() {
            println!();
            println!("  Safe zones:");
            for z in &contract.safe_zones {
                println!("    ✓ {} — {}", z.pattern, z.reason);
            }
        }

        if !contract.protected.is_empty() {
            println!();
            println!("  Protected:");
            for z in &contract.protected {
                println!("    ⚠ {} — {}", z.pattern, z.reason);
            }
        }

        if !contract.validation.is_empty() {
            println!();
            println!("  Validation:");
            for v in &contract.validation {
                let req = if v.required { " (required)" } else { "" };
                println!("    $ {} — {}{}", v.command, v.name, req);
            }
        }

        if !contract.ownership.is_empty() {
            println!();
            println!("  Ownership:");
            for o in &contract.ownership {
                println!("    {} -> {}", o.pattern, o.owner);
            }
        }
    }
    Ok(())
}
