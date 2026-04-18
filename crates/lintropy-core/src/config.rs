//! Minimal config/query types that the engine consumes.

use std::path::PathBuf;

use tree_sitter::Query;

use crate::{
    predicates::{parse_general_predicates, parse_general_predicates_by_pattern, CustomPredicate},
    types::{RuleId, Severity}, Result,
};

/// Placeholder for root settings owned by the future config loader.
#[derive(Debug, Clone, Default)]
pub struct Settings;

/// Loaded lintropy configuration.
#[derive(Debug, Default)]
pub struct Config {
    pub settings: Settings,
    pub rules: Vec<RuleConfig>,
}

/// A loaded rule with enough information for the engine to execute it.
#[derive(Debug)]
pub struct RuleConfig {
    pub id: RuleId,
    pub severity: Severity,
    pub message: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub tags: Vec<String>,
    pub docs_url: Option<String>,
    pub language: Option<String>,
    pub kind: RuleKind,
    pub fix: Option<String>,
    pub source_path: PathBuf,
}

/// Rule discriminator.
#[derive(Debug)]
pub enum RuleKind {
    Query(QueryRule),
    Match(MatchRule),
}

/// A compiled tree-sitter query rule.
#[derive(Debug)]
pub struct QueryRule {
    pub source: String,
    pub query: Query,
    pub predicates: Vec<CustomPredicate>,
    pub predicates_by_pattern: Vec<Vec<CustomPredicate>>,
}

impl QueryRule {
    pub fn new(source: impl Into<String>, query: Query) -> Result<Self> {
        let predicates = parse_general_predicates(&query)?;
        let predicates_by_pattern = parse_general_predicates_by_pattern(&query)?;
        Ok(Self {
            source: source.into(),
            query,
            predicates,
            predicates_by_pattern,
        })
    }
}

/// Placeholder for regex rules; WP2 does not execute them.
#[derive(Debug, Clone, Default)]
pub struct MatchRule;

impl RuleConfig {
    pub fn query_rule(&self) -> Option<&QueryRule> {
        match &self.kind {
            RuleKind::Query(rule) => Some(rule),
            RuleKind::Match(_) => None,
        }
    }
}
