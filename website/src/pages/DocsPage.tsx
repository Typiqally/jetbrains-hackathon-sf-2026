import { useEffect, useRef, useState } from "react";
import { Link, useLocation } from "react-router-dom";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";
import rehypeSlug from "rehype-slug";
import { getDoc } from "../docs";
import { findLeaf, neighbors } from "../nav";
import Sidebar from "../components/Sidebar";
import Toc, { type TocItem } from "../components/Toc";
import NotFound from "./NotFound";

export default function DocsPage() {
  const location = useLocation();
  const slug = location.pathname.replace(/^\/+/, "").replace(/\/+$/, "");
  const doc = getDoc(slug);
  const contentRef = useRef<HTMLDivElement>(null);
  const [tocItems, setTocItems] = useState<TocItem[]>([]);

  useEffect(() => {
    if (!contentRef.current) return;
    const hs = contentRef.current.querySelectorAll("h2, h3");
    const items: TocItem[] = Array.from(hs)
      .filter((h) => h.id)
      .map((h) => ({
        id: h.id,
        text: h.textContent ?? "",
        level: h.tagName === "H2" ? 2 : 3,
      }));
    setTocItems(items);
  }, [slug, doc?.body]);

  useEffect(() => {
    if (location.hash) {
      const el = document.getElementById(location.hash.slice(1));
      if (el) el.scrollIntoView({ behavior: "smooth", block: "start" });
    } else {
      window.scrollTo({ top: 0, behavior: "instant" as ScrollBehavior });
    }
  }, [slug, location.hash]);

  if (!doc) return <NotFound />;

  const leafInfo = findLeaf(slug);
  const { prev, next } = neighbors(slug);
  const title = doc.title ?? leafInfo?.leaf.title ?? "Docs";

  return (
    <div className="mx-auto w-full max-w-6xl px-4 sm:px-6">
      <div className="flex gap-10">
        <Sidebar />
        <article className="min-w-0 flex-1 py-8 sm:py-10" data-pagefind-body>
          {leafInfo && (
            <p className="mb-2 text-[11px] font-bold uppercase tracking-[0.18em] text-[color:var(--color-fg-subtle)]">
              {leafInfo.group.title}
            </p>
          )}
          <div className="prose" ref={contentRef}>
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              rehypePlugins={[
                rehypeSlug,
                [rehypeHighlight, { detect: true, ignoreMissing: true }],
              ]}
              components={{
                a: ({ href, children, ...props }) => {
                  const url = href ?? "";
                  if (/^(https?:|mailto:|#)/.test(url)) {
                    const external = url.startsWith("http");
                    return (
                      <a
                        href={url}
                        {...props}
                        target={external ? "_blank" : undefined}
                        rel={external ? "noopener noreferrer" : undefined}
                      >
                        {children}
                      </a>
                    );
                  }
                  const resolved = resolveInternalLink(slug, url);
                  return (
                    <Link to={resolved} {...props}>
                      {children}
                    </Link>
                  );
                },
              }}
            >
              {injectTitle(title, doc.body)}
            </ReactMarkdown>
          </div>

          <nav className="mt-14 grid gap-3 border-t border-[color:var(--color-border)] pt-6 sm:grid-cols-2">
            {prev ? (
              <Link
                to={`/${prev.slug}`}
                className="group rounded-xl border border-[color:var(--color-border)] bg-white/[0.02] p-4 transition hover:border-[color:var(--color-border-strong)]"
              >
                <p className="text-[11px] font-bold uppercase tracking-[0.15em] text-[color:var(--color-fg-subtle)]">
                  Previous
                </p>
                <p className="mt-1 text-sm text-[color:var(--color-fg)] group-hover:text-[color:var(--color-accent)]">
                  {prev.title}
                </p>
              </Link>
            ) : (
              <span />
            )}
            {next ? (
              <Link
                to={`/${next.slug}`}
                className="group rounded-xl border border-[color:var(--color-border)] bg-white/[0.02] p-4 text-right transition hover:border-[color:var(--color-border-strong)]"
              >
                <p className="text-[11px] font-bold uppercase tracking-[0.15em] text-[color:var(--color-fg-subtle)]">
                  Next
                </p>
                <p className="mt-1 text-sm text-[color:var(--color-fg)] group-hover:text-[color:var(--color-accent)]">
                  {next.title}
                </p>
              </Link>
            ) : (
              <span />
            )}
          </nav>
        </article>
        <Toc items={tocItems} />
      </div>
    </div>
  );
}

function injectTitle(title: string, body: string): string {
  return /^#\s+/m.test(body) ? body : `# ${title}\n\n${body}`;
}

function resolveInternalLink(fromSlug: string, href: string): string {
  const [pathPart, hash] = href.split("#");
  if (!pathPart) return `#${hash ?? ""}`;
  const baseDir = fromSlug.includes("/") ? fromSlug.replace(/\/[^/]+$/, "") : "";
  const parts = (baseDir ? baseDir.split("/") : []).concat(pathPart.split("/"));
  const resolved: string[] = [];
  for (const p of parts) {
    if (p === "" || p === ".") continue;
    if (p === "..") resolved.pop();
    else resolved.push(p);
  }
  const cleaned = resolved
    .join("/")
    .replace(/\.md$/, "")
    .replace(/\/index$/, "");
  return `/${cleaned}${hash ? `#${hash}` : ""}`;
}
