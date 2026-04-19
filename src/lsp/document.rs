//! In-memory document store for the LSP server.
//!
//! The client pushes the authoritative buffer state (open, change, close);
//! we keep the latest text + version indexed by URI so the engine can lint
//! the live buffer instead of whatever is on disk.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tower_lsp::lsp_types::{Range, Url};
use tree_sitter::Tree;

use crate::langs::Language;

use super::position::{apply_change, compute_input_edit};

/// Cached tree-sitter parse for one open buffer.
///
/// Held between edits so subsequent parses can reuse the old tree via
/// `Parser::parse(src, Some(&tree))`. `language` is stored because a
/// single buffer path may theoretically map to different grammars
/// across rule reloads — if the detected language changes we drop the
/// cache rather than try to reuse a tree built against the wrong
/// grammar.
#[derive(Debug, Clone)]
pub struct CachedParse {
    pub language: Language,
    pub tree: Tree,
}

/// Latest known state of a single open editor buffer.
///
/// `text` is stored as `Arc<String>` so handlers can cheaply snapshot
/// it (`Arc::clone`) before releasing the state mutex, and long-running
/// lint passes see a stable view even while new edits come in. On write
/// we call `Arc::make_mut` — cheap (refcount == 1) when no snapshot is
/// outstanding.
#[derive(Debug, Clone)]
pub struct Document {
    /// Filesystem path derived from the URI. `lint_buffer` needs it for
    /// language detection (via extension) and include/exclude glob matching.
    pub path: PathBuf,
    /// Buffer contents, UTF-8.
    pub text: Arc<String>,
    /// Monotonic version from the client.
    pub version: i32,
    /// Last parse of this buffer, with tree-sitter edits applied to
    /// track the text mutations since it was produced. `None` until the
    /// first lint pass, or after a full-buffer replace.
    pub parse: Option<CachedParse>,
}

/// Map from `textDocument.uri` to the latest [`Document`].
#[derive(Debug, Default)]
pub struct DocumentStore {
    docs: HashMap<Url, Document>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the full buffer for `uri` (`didOpen`).
    pub fn set(&mut self, uri: Url, text: String, version: i32) {
        let path = uri_to_path(&uri).unwrap_or_else(|| PathBuf::from(uri.path()));
        self.docs.insert(
            uri,
            Document {
                path,
                text: Arc::new(text),
                version,
                parse: None,
            },
        );
    }

    /// Apply one incremental edit from a `didChange` notification.
    ///
    /// `range == None` is the full-sync fallback (some clients still send
    /// it even with incremental negotiated); `range == Some(..)` patches
    /// only that UTF-16 range.
    ///
    /// No-op if the URI isn't tracked (shouldn't happen — the client
    /// always opens before changing — but we'd rather drop the edit than
    /// synthesize a partial document from scratch).
    pub fn apply_edit(&mut self, uri: &Url, range: Option<Range>, new_text: &str, version: i32) {
        if let Some(doc) = self.docs.get_mut(uri) {
            match range {
                None => {
                    // Full replace — any cached parse is against a
                    // buffer that no longer exists.
                    *Arc::make_mut(&mut doc.text) = new_text.to_string();
                    doc.parse = None;
                }
                Some(range) => {
                    if let Some(parse) = &mut doc.parse {
                        let edit = compute_input_edit(&doc.text, range, new_text);
                        parse.tree.edit(&edit);
                    }
                    apply_change(Arc::make_mut(&mut doc.text), Some(range), new_text);
                }
            }
            doc.version = version;
        }
    }

    /// Update the cached parse for `uri` iff the buffer is still at
    /// `version`. Discards the incoming tree when a newer edit has
    /// raced ahead of the lint pass that produced it.
    pub fn store_parse(&mut self, uri: &Url, version: i32, parse: CachedParse) {
        if let Some(doc) = self.docs.get_mut(uri) {
            if doc.version == version {
                doc.parse = Some(parse);
            }
        }
    }

    pub fn get(&self, uri: &Url) -> Option<&Document> {
        self.docs.get(uri)
    }

    pub fn remove(&mut self, uri: &Url) {
        self.docs.remove(uri);
    }

    /// Iterate over every currently open buffer. Used when the config
    /// reloads and we need to re-lint all buffers.
    pub fn iter(&self) -> impl Iterator<Item = (&Url, &Document)> {
        self.docs.iter()
    }
}

fn uri_to_path(uri: &Url) -> Option<PathBuf> {
    if uri.scheme() != "file" {
        return None;
    }
    uri.to_file_path().ok().map(|p| p as PathBuf)
}

/// Turn a filesystem path back into a `file://` URI. Used when publishing
/// diagnostics for a path the server knows only by filesystem path
/// (e.g. workspace scan triggered by config reload).
#[allow(dead_code)]
pub fn path_to_uri(path: &Path) -> Option<Url> {
    Url::from_file_path(path).ok()
}
