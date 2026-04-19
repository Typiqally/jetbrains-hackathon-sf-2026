//! `tower_lsp::LanguageServer` implementation — the glue between the
//! protocol and the lintropy engine.
//!
//! State model:
//! - configs are cached per discovered `lintropy.yaml` root, so nested
//!   projects get isolated rule contexts instead of inheriting the
//!   workspace root's rules.
//! - `documents` tracks the client's authoritative buffer state.
//! - `PreparedRules` is rebuilt per lint. Glob compile is cheap and the
//!   alternative (self-referential `PreparedRules` borrowing from `Config`)
//!   is not worth the complexity.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result as JsonRpcResult;
use tower_lsp::lsp_types::{
    CodeActionOrCommand, CodeActionParams, CodeActionProviderCapability, CodeActionResponse,
    CompletionOptions, CompletionParams, CompletionResponse, DidChangeConfigurationParams,
    DidChangeTextDocumentParams, DidChangeWatchedFilesParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, InitializeParams, InitializeResult,
    InitializedParams, MessageType, SemanticTokens, SemanticTokensFullOptions,
    SemanticTokensOptions, SemanticTokensParams, SemanticTokensResult,
    SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url, WorkDoneProgressOptions,
};
use tower_lsp::{Client, LanguageServer};

use crate::core::{discovery, Config, PreparedRules};

fn is_rule_file(path: &std::path::Path) -> bool {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    name == "lintropy.yaml"
        || name.ends_with(".rule.yaml")
        || name.ends_with(".rule.yml")
        || name.ends_with(".rules.yaml")
        || name.ends_with(".rules.yml")
}

use super::actions::{quickfix_for, ranges_intersect};
use super::diagnostics::to_lsp;
use super::document::DocumentStore;
use super::{completion, rule_lint, semantic_tokens};

/// Shared LSP backend.
///
/// Wraps all mutable state + the `Client` in an `Arc<Inner>` so cheap
/// handles can be passed to spawned tasks without depending on
/// tower-lsp's internal lifetime management.
pub struct Backend {
    inner: Arc<Inner>,
}

struct Inner {
    client: Client,
    state: Mutex<State>,
}

struct State {
    /// Cached configs keyed by the specific `lintropy.yaml` that owns
    /// a document subtree. This lets nested projects shadow parent
    /// workspaces while still merging `.lintropy/` files inside the
    /// resolved context.
    configs: HashMap<PathBuf, CachedConfig>,
    documents: DocumentStore,
    /// Cached semantic tokens keyed by doc version. Cleared on version
    /// bump; a single keystroke therefore invalidates exactly once.
    semantic_tokens_cache: HashMap<Url, (i32, Arc<SemanticTokens>)>,
}

#[derive(Clone)]
enum CachedConfig {
    Loaded {
        config: Arc<Config>,
        loaded_at: SystemTime,
    },
    Failed,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            inner: Arc::new(Inner {
                client,
                state: Mutex::new(State {
                    configs: HashMap::new(),
                    documents: DocumentStore::new(),
                    semantic_tokens_cache: HashMap::new(),
                }),
            }),
        }
    }

    fn client(&self) -> &Client {
        &self.inner.client
    }

    fn state_mutex(&self) -> &Mutex<State> {
        &self.inner.state
    }

    async fn log(&self, ty: MessageType, message: impl Into<String>) {
        self.inner.client.log_message(ty, message.into()).await;
    }

    async fn clear_config_cache(&self) {
        let mut state = self.state_mutex().lock().await;
        state.configs.clear();
    }

    /// Resolve the nearest `lintropy.yaml` for `path`, loading and
    /// caching that context on demand.
    async fn config_for_path(&self, path: &std::path::Path) -> Option<Arc<Config>> {
        let start = if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent()
                .map(std::path::Path::to_path_buf)
                .unwrap_or_else(|| path.to_path_buf())
        };
        let root_config = discovery::find_root_config(&start).ok()?;

        let cached = {
            let state = self.state_mutex().lock().await;
            state.configs.get(&root_config).cloned()
        };
        match cached {
            Some(CachedConfig::Loaded { config, .. }) => return Some(config),
            Some(CachedConfig::Failed) => return None,
            None => {}
        }

        match Config::load_from_path(&root_config) {
            Ok(config) => {
                self.log(
                    MessageType::INFO,
                    format!(
                        "lintropy: loaded {} rules from {}",
                        config.rules.len(),
                        config.root_dir.display()
                    ),
                )
                .await;
                let config = Arc::new(config);
                let mut state = self.state_mutex().lock().await;
                state.configs.insert(
                    root_config,
                    CachedConfig::Loaded {
                        config: config.clone(),
                        loaded_at: SystemTime::now(),
                    },
                );
                Some(config)
            }
            Err(err) => {
                self.log(
                    MessageType::ERROR,
                    format!("lintropy: config load failed: {err}"),
                )
                .await;
                let mut state = self.state_mutex().lock().await;
                state.configs.insert(root_config, CachedConfig::Failed);
                None
            }
        }
    }

    /// Drop cached configs whose backing rule files have changed on disk
    /// since the last load. Returns `true` if anything was invalidated
    /// (i.e. callers should `republish_all`).
    ///
    /// Compares the saved file's mtime against each cached config's
    /// `loaded_at`: cheaper than re-reading YAML and stops the
    /// "save no-op → full workspace relint" ping-pong.
    async fn invalidate_stale_configs(&self, changed_file: &std::path::Path) -> bool {
        let mtime = match std::fs::metadata(changed_file).and_then(|m| m.modified()) {
            Ok(t) => t,
            Err(_) => {
                // File disappeared or metadata unavailable — be safe,
                // drop everything so the next lookup reloads.
                let mut state = self.state_mutex().lock().await;
                let had = !state.configs.is_empty();
                state.configs.clear();
                return had;
            }
        };
        let mut state = self.state_mutex().lock().await;
        let mut dirty = false;
        state.configs.retain(|_root, cached| match cached {
            CachedConfig::Loaded { loaded_at, .. } => {
                if mtime > *loaded_at {
                    dirty = true;
                    false
                } else {
                    true
                }
            }
            // Retry failed configs on any rule-file save — maybe the
            // user is in the middle of fixing a syntax error.
            CachedConfig::Failed => {
                dirty = true;
                false
            }
        });
        dirty
    }

    /// Re-lint every open buffer and publish diagnostics. Used after
    /// config reload; for single-buffer updates call [`publish_for`].
    async fn republish_all(&self) {
        let snapshot: Vec<_> = {
            let state = self.state_mutex().lock().await;
            state
                .documents
                .iter()
                .map(|(uri, doc)| {
                    (
                        uri.clone(),
                        doc.version,
                        doc.text.clone(),
                        doc.path.clone(),
                        doc.parse.clone(),
                    )
                })
                .collect()
        };
        for (uri, version, text, path, parse) in snapshot {
            self.publish_with(uri, version, &text, &path, parse).await;
        }
    }

    /// Lint `uri`'s current buffer and publish diagnostics.
    async fn publish_for(&self, uri: &Url) {
        let snapshot = {
            let state = self.state_mutex().lock().await;
            state
                .documents
                .get(uri)
                .map(|d| (d.version, d.text.clone(), d.path.clone(), d.parse.clone()))
        };
        if let Some((version, text, path, parse)) = snapshot {
            self.publish_with(uri.clone(), version, &text, &path, parse)
                .await;
        }
    }

    async fn publish_with(
        &self,
        uri: Url,
        version: i32,
        text: &str,
        path: &std::path::Path,
        parse: Option<super::document::CachedParse>,
    ) {
        let lsp_diags = if is_rule_file(path) {
            rule_lint::lint(path, text)
        } else {
            match self.lint(&uri, version, text, path, parse).await {
                Some(diags) => diags
                    .iter()
                    .map(|d| to_lsp(d, text, None))
                    .collect::<Vec<_>>(),
                None => Vec::new(),
            }
        };
        self.client()
            .publish_diagnostics(uri, lsp_diags, Some(version))
            .await;
    }

    /// Core engine invocation: resolve the nearest config for `path`,
    /// reuse the cached parse tree if available, and lint the buffer.
    ///
    /// Splits engine work into `parse_buffer` (incremental, reusing the
    /// prior tree if present) + `run_queries` (pure query traversal) so
    /// a single keystroke pays only the reparse delta, not a full parse.
    ///
    /// The freshly parsed tree is stored back under the document if the
    /// buffer is still at `version` when we finish — if a newer edit
    /// raced in, we drop the tree since the document's existing
    /// `parse.tree` has received `tree.edit` updates we did not apply.
    async fn lint(
        &self,
        uri: &Url,
        version: i32,
        text: &str,
        path: &std::path::Path,
        parse: Option<super::document::CachedParse>,
    ) -> Option<Vec<crate::core::Diagnostic>> {
        let config = self.config_for_path(path).await?;
        let prepared = PreparedRules::prepare(config.as_ref()).ok()?;
        let src = text.as_bytes();
        // Only reuse the cached tree if it was parsed against the same
        // grammar we'd pick now — guards against a rename that flips
        // the file's language (e.g. `.ts` → `.tsx`).
        let detected = crate::langs::language_from_path(path);
        let old_tree_ref = parse
            .as_ref()
            .filter(|p| Some(p.language) == detected)
            .map(|p| &p.tree);
        let (language, tree) = prepared.parse_buffer(path, src, old_tree_ref).ok()??;
        let diagnostics = prepared.run_queries(path, src, language, &tree).ok()?;
        {
            let mut state = self.state_mutex().lock().await;
            state.documents.store_parse(
                uri,
                version,
                super::document::CachedParse { language, tree },
            );
        }
        Some(diagnostics)
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> JsonRpcResult<InitializeResult> {
        let _ = params;

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    // `@` / `#` / `(` → query context; `{` → template
                    // interpolation; `:` / space → `language:` value.
                    // Clients still send completion on Ctrl-Space, so
                    // coverage is additive.
                    trigger_characters: Some(vec![
                        "@".into(),
                        "#".into(),
                        "(".into(),
                        "{".into(),
                        ":".into(),
                        " ".into(),
                    ]),
                    resolve_provider: Some(false),
                    ..CompletionOptions::default()
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: semantic_tokens::legend(),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: None,
                            work_done_progress_options: WorkDoneProgressOptions::default(),
                        },
                    ),
                ),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "lintropy".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> JsonRpcResult<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        // Fast path: tokens are stable per `(uri, version)` — return the
        // cached result without re-scanning if nothing has changed since
        // the last request.
        let (version, text) = {
            let state = self.state_mutex().lock().await;
            if let Some((ver, tokens)) = state.semantic_tokens_cache.get(&uri) {
                if let Some(doc) = state.documents.get(&uri) {
                    if doc.version == *ver {
                        return Ok(Some(SemanticTokensResult::Tokens((**tokens).clone())));
                    }
                }
            }
            match state.documents.get(&uri) {
                Some(doc) => (doc.version, doc.text.clone()),
                None => return Ok(None),
            }
        };
        let tokens = match semantic_tokens::tokenize(&text) {
            Some(t) => t,
            None => return Ok(None),
        };
        {
            let mut state = self.state_mutex().lock().await;
            state
                .semantic_tokens_cache
                .insert(uri, (version, Arc::new(tokens.clone())));
        }
        Ok(Some(SemanticTokensResult::Tokens(tokens)))
    }

    async fn initialized(&self, _: InitializedParams) {
        self.clear_config_cache().await;
    }

    async fn shutdown(&self) -> JsonRpcResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        {
            let mut state = self.state_mutex().lock().await;
            state.documents.set(doc.uri.clone(), doc.text, doc.version);
        }
        self.publish_for(&doc.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;
        // `TextDocumentSyncKind::INCREMENTAL`: `content_changes` is an
        // ordered list of edits that must be applied in sequence. Each
        // `range == None` is a full-buffer replace; each `range == Some`
        // patches only that UTF-16 range.
        {
            let mut state = self.state_mutex().lock().await;
            for change in params.content_changes {
                state
                    .documents
                    .apply_edit(&uri, change.range, &change.text, version);
            }
            // Version bumped; cached semantic tokens are now stale.
            state.semantic_tokens_cache.remove(&uri);
        }
        self.publish_for(&uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let path = params.text_document.uri.to_file_path().ok();
        let is_config_file = path.as_deref().is_some_and(is_rule_file);
        if is_config_file {
            let changed = path.expect("is_config_file implies path");
            if self.invalidate_stale_configs(&changed).await {
                self.republish_all().await;
            } else {
                // File save with no on-disk change (touch-save, autosave
                // of an unmodified buffer). Re-publish just this buffer
                // against the unchanged config.
                self.publish_for(&params.text_document.uri).await;
            }
        } else {
            // Nothing to do — we lint on didChange. Republish just in case
            // the client flushed a partial state between changes.
            self.publish_for(&params.text_document.uri).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        {
            let mut state = self.state_mutex().lock().await;
            state.documents.remove(&uri);
            state.semantic_tokens_cache.remove(&uri);
        }
        // Clear diagnostics so the editor doesn't keep them after close.
        self.client()
            .publish_diagnostics(uri, Vec::new(), None)
            .await;
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.clear_config_cache().await;
        self.republish_all().await;
    }

    async fn did_change_watched_files(&self, _params: DidChangeWatchedFilesParams) {
        self.clear_config_cache().await;
        self.republish_all().await;
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> JsonRpcResult<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let (text, path) = {
            let state = self.state_mutex().lock().await;
            match state.documents.get(&uri) {
                Some(doc) => (doc.text.clone(), doc.path.clone()),
                None => return Ok(None),
            }
        };
        let items = completion::complete(&path, &text, params.text_document_position.position);
        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CompletionResponse::Array(items)))
        }
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> JsonRpcResult<Option<CodeActionResponse>> {
        let uri = params.text_document.uri.clone();
        let (version, text, path, parse) = {
            let state = self.state_mutex().lock().await;
            match state.documents.get(&uri) {
                Some(doc) => (
                    doc.version,
                    doc.text.clone(),
                    doc.path.clone(),
                    doc.parse.clone(),
                ),
                None => return Ok(None),
            }
        };

        let diagnostics = match self.lint(&uri, version, &text, &path, parse).await {
            Some(d) => d,
            None => return Ok(None),
        };

        let requested_range = params.range;
        let mut actions: Vec<CodeActionOrCommand> = Vec::new();
        for diag in &diagnostics {
            let Some(action) = quickfix_for(&uri, &text, diag) else {
                continue;
            };
            let diag_range = action
                .diagnostics
                .as_ref()
                .and_then(|d| d.first())
                .map(|d| d.range)
                .unwrap_or(requested_range);
            if !ranges_intersect(diag_range, requested_range) {
                continue;
            }
            actions.push(CodeActionOrCommand::CodeAction(action));
        }

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }
}
