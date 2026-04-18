//! `lintropy rules` — list every loaded rule.

use lintropy_core::{Config, RuleConfig, RuleKind};
use serde_json::json;

use crate::cli::{OutputFormat, RulesArgs};
use crate::commands::{load_config, print_warnings};
use crate::exit::{CliError, EXIT_OK};

pub fn run(args: RulesArgs) -> Result<u8, CliError> {
    let config = load_config(args.config.as_deref())?;
    print_warnings(&config);

    match args.format {
        OutputFormat::Text => print_text(&config),
        OutputFormat::Json => print_json(&config)?,
    }
    Ok(EXIT_OK)
}

fn print_text(config: &Config) {
    for rule in &config.rules {
        println!(
            "{} [{}] {}",
            rule.id,
            severity_label(rule.severity),
            rule.source_path.display()
        );
    }
}

fn print_json(config: &Config) -> Result<(), CliError> {
    let array: Vec<_> = config.rules.iter().map(rule_to_json).collect();
    let json = serde_json::to_string_pretty(&array)
        .map_err(|err| CliError::internal(format!("json: {err}")))?;
    println!("{json}");
    Ok(())
}

fn rule_to_json(rule: &RuleConfig) -> serde_json::Value {
    let kind = match &rule.kind {
        RuleKind::Query(_) => "query",
        RuleKind::Match(_) => "match",
    };
    json!({
        "id": rule.id.as_str(),
        "severity": severity_label(rule.severity),
        "language": rule.language.map(|l| l.name()),
        "kind": kind,
        "source_path": rule.source_path.display().to_string(),
        "tags": rule.tags,
        "docs_url": rule.docs_url,
        "include": rule.include,
        "exclude": rule.exclude,
    })
}

fn severity_label(severity: lintropy_core::Severity) -> &'static str {
    match severity {
        lintropy_core::Severity::Error => "error",
        lintropy_core::Severity::Warning => "warning",
        lintropy_core::Severity::Info => "info",
    }
}
