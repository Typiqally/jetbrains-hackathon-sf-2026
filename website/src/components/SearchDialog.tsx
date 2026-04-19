import { useEffect, useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { Search as SearchIcon, X } from "lucide-react";
import { allDocs } from "../docs";
import { findLeaf } from "../nav";

type Hit = {
  slug: string;
  title: string;
  group: string;
  excerpt: Array<{ text: string; hit: boolean }>;
  score: number;
};

export default function SearchDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (v: boolean) => void;
}) {
  const [query, setQuery] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  const docs = useMemo(() => indexDocs(), []);

  useEffect(() => {
    if (!open) return;
    setQuery("");
    setTimeout(() => inputRef.current?.focus(), 0);
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onOpenChange(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onOpenChange]);

  const hits = useMemo(() => (query.trim() ? search(docs, query) : []), [docs, query]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/60 p-4 pt-[15vh]"
      onClick={() => onOpenChange(false)}
    >
      <div
        className="w-full max-w-xl overflow-hidden rounded-2xl border border-[color:var(--color-border)] bg-[color:var(--color-surface-2)] shadow-[0_30px_80px_rgba(0,0,0,0.5)]"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-2 border-b border-[color:var(--color-border)] px-4 py-3">
          <SearchIcon size={16} className="text-[color:var(--color-fg-subtle)]" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search docs…"
            className="flex-1 bg-transparent text-sm text-[color:var(--color-fg)] outline-none placeholder:text-[color:var(--color-fg-faint)]"
          />
          <button
            type="button"
            onClick={() => onOpenChange(false)}
            className="text-[color:var(--color-fg-subtle)] transition hover:text-[color:var(--color-fg)]"
            aria-label="Close"
          >
            <X size={16} />
          </button>
        </div>

        <div className="max-h-[50vh] overflow-y-auto">
          {!query && (
            <p className="px-4 py-6 text-center text-xs text-[color:var(--color-fg-subtle)]">
              Start typing to search.
            </p>
          )}
          {query && hits.length === 0 && (
            <p className="px-4 py-6 text-center text-xs text-[color:var(--color-fg-subtle)]">
              No results for "{query}".
            </p>
          )}
          {hits.length > 0 && (
            <ul className="divide-y divide-[color:var(--color-border)]">
              {hits.map((hit) => (
                <li key={hit.slug}>
                  <Link
                    to={`/${hit.slug}`}
                    onClick={() => onOpenChange(false)}
                    className="block px-4 py-3 transition hover:bg-white/5"
                  >
                    <p className="text-[10px] font-bold uppercase tracking-[0.15em] text-[color:var(--color-fg-subtle)]">
                      {hit.group}
                    </p>
                    <p className="mt-1 text-sm font-medium text-[color:var(--color-fg)]">
                      {hit.title}
                    </p>
                    {hit.excerpt.length > 0 && (
                      <p className="mt-1 text-xs text-[color:var(--color-fg-muted)]">
                        {hit.excerpt.map((seg, idx) =>
                          seg.hit ? (
                            <mark
                              key={idx}
                              className="bg-[rgba(141,225,180,0.25)] text-[color:var(--color-accent)]"
                            >
                              {seg.text}
                            </mark>
                          ) : (
                            <span key={idx}>{seg.text}</span>
                          ),
                        )}
                      </p>
                    )}
                  </Link>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </div>
  );
}

type IndexedDoc = {
  slug: string;
  title: string;
  group: string;
  plain: string;
  plainLower: string;
  titleLower: string;
};

function indexDocs(): IndexedDoc[] {
  return allDocs()
    .filter((d) => d.slug !== "")
    .map((d) => {
      const leafInfo = findLeaf(d.slug);
      const title = d.title ?? leafInfo?.leaf.title ?? d.slug;
      const group = leafInfo?.group.title ?? "Docs";
      const plain = stripMarkdown(d.body);
      return {
        slug: d.slug,
        title,
        group,
        plain,
        plainLower: plain.toLowerCase(),
        titleLower: title.toLowerCase(),
      };
    });
}

function stripMarkdown(src: string): string {
  return src
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/`([^`]*)`/g, "$1")
    .replace(/!\[[^\]]*\]\([^)]*\)/g, " ")
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .replace(/^#{1,6}\s+/gm, "")
    .replace(/[*_~>]/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

function search(docs: IndexedDoc[], query: string): Hit[] {
  const terms = query
    .toLowerCase()
    .split(/\s+/)
    .filter((t) => t.length > 1);
  if (!terms.length) return [];

  const scored: Hit[] = [];
  for (const d of docs) {
    let score = 0;
    for (const t of terms) {
      const inTitle = d.titleLower.includes(t);
      const bodyCount = countOccurrences(d.plainLower, t);
      if (inTitle) score += 10;
      score += bodyCount;
    }
    if (score > 0) {
      scored.push({
        slug: d.slug,
        title: d.title,
        group: d.group,
        excerpt: buildExcerpt(d.plain, terms),
        score,
      });
    }
  }

  scored.sort((a, b) => b.score - a.score);
  return scored.slice(0, 8);
}

function countOccurrences(haystack: string, needle: string): number {
  if (!needle) return 0;
  let count = 0;
  let idx = 0;
  while ((idx = haystack.indexOf(needle, idx)) !== -1) {
    count += 1;
    idx += needle.length;
  }
  return count;
}

function buildExcerpt(
  plain: string,
  terms: string[],
): Array<{ text: string; hit: boolean }> {
  const lower = plain.toLowerCase();
  let firstIdx = -1;
  let firstTerm: string | null = null;
  for (const t of terms) {
    const idx = lower.indexOf(t);
    if (idx >= 0 && (firstIdx === -1 || idx < firstIdx)) {
      firstIdx = idx;
      firstTerm = t;
    }
  }
  if (firstIdx === -1 || !firstTerm) return [];

  const window = 80;
  const start = Math.max(0, firstIdx - window);
  const end = Math.min(plain.length, firstIdx + firstTerm.length + window);
  const slice = plain.slice(start, end);

  const sliceLower = slice.toLowerCase();
  const segments: Array<{ text: string; hit: boolean }> = [];
  let cursor = 0;
  while (cursor < slice.length) {
    let nextHit = -1;
    let nextTerm = "";
    for (const t of terms) {
      const idx = sliceLower.indexOf(t, cursor);
      if (idx >= 0 && (nextHit === -1 || idx < nextHit)) {
        nextHit = idx;
        nextTerm = t;
      }
    }
    if (nextHit === -1) {
      segments.push({ text: slice.slice(cursor), hit: false });
      break;
    }
    if (nextHit > cursor) {
      segments.push({ text: slice.slice(cursor, nextHit), hit: false });
    }
    segments.push({
      text: slice.slice(nextHit, nextHit + nextTerm.length),
      hit: true,
    });
    cursor = nextHit + nextTerm.length;
  }

  const prefix = start > 0 ? "…" : "";
  const suffix = end < plain.length ? "…" : "";
  const head: Array<{ text: string; hit: boolean }> = prefix
    ? [{ text: prefix, hit: false }]
    : [];
  const tail: Array<{ text: string; hit: boolean }> = suffix
    ? [{ text: suffix, hit: false }]
    : [];
  return [...head, ...segments, ...tail];
}
