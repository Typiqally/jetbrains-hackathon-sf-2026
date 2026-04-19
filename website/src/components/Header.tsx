import { useEffect, useState } from "react";
import { Link, NavLink, useLocation } from "react-router-dom";
import { Menu, Search, X } from "lucide-react";
import { primaryNav, nav } from "../nav";
import SearchDialog from "./SearchDialog";

function isActiveSection(to: string, path: string): boolean {
  if (to === "/") return path === "/";
  // Match the first path segment so that, e.g., /configuration counts as
  // Reference active even though the link itself points at /configuration.
  const seg = (p: string) => p.split("/").filter(Boolean)[0] ?? "";
  const toSeg = seg(to);
  if (toSeg === "integrations") return path.startsWith("/integrations");
  if (toSeg === "overview" || toSeg === "getting-started")
    return ["overview", "getting-started"].includes(seg(path));
  if (toSeg === "configuration" || toSeg === "rule-language" || toSeg === "cli")
    return ["configuration", "rule-language", "cli"].includes(seg(path));
  if (toSeg === "troubleshooting") return seg(path) === "troubleshooting";
  return false;
}

export default function Header() {
  const { pathname } = useLocation();
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);

  useEffect(() => setDrawerOpen(false), [pathname]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setSearchOpen(true);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <>
      <header className="sticky top-0 z-40 border-b border-[color:var(--color-border)] backdrop-blur-xl backdrop-saturate-150 bg-[rgba(8,8,8,0.55)]">
        <div className="mx-auto flex h-14 max-w-6xl items-center gap-3 px-4 sm:px-6">
          <Link
            to="/"
            className="text-[15px] font-bold tracking-tight text-[color:var(--color-fg)] transition hover:text-[color:var(--color-accent)]"
          >
            Lintropy
          </Link>

          <nav className="hidden items-center gap-1 border-l border-[color:var(--color-border)] pl-3 md:flex">
            {primaryNav.map((item) => {
              const active = isActiveSection(item.to, pathname);
              return (
                <Link
                  key={item.to}
                  to={item.to}
                  className={`rounded-full px-3 py-1.5 text-[13px] font-medium transition ${
                    active
                      ? "bg-[rgba(141,225,180,0.08)] text-[color:var(--color-accent)]"
                      : "text-[color:var(--color-fg-muted)] hover:bg-white/5 hover:text-[color:var(--color-fg)]"
                  }`}
                >
                  {item.title}
                </Link>
              );
            })}
          </nav>

          <div className="flex-1" />

          <button
            type="button"
            onClick={() => setSearchOpen(true)}
            className="hidden items-center gap-2 rounded-full border border-[color:var(--color-border)] bg-white/[0.02] px-3 py-1.5 text-xs text-[color:var(--color-fg-subtle)] transition hover:border-[color:var(--color-border-strong)] hover:text-[color:var(--color-fg)] sm:flex"
            aria-label="Search"
          >
            <Search size={14} />
            <span>Search</span>
            <kbd className="rounded border border-[color:var(--color-border)] px-1.5 py-0.5 text-[10px] text-[color:var(--color-fg-faint)]">
              ⌘K
            </kbd>
          </button>

          <a
            href="https://github.com/Typiqally/lintropy"
            className="hidden rounded-full border border-[color:var(--color-border)] bg-white/[0.02] px-3 py-1.5 text-xs font-semibold text-[color:var(--color-fg-muted)] transition hover:border-[color:var(--color-border-strong)] hover:text-[color:var(--color-fg)] sm:inline-flex"
          >
            GitHub
          </a>

          <button
            type="button"
            onClick={() => setDrawerOpen((v) => !v)}
            className="inline-flex h-9 w-9 items-center justify-center rounded-md text-[color:var(--color-fg-muted)] transition hover:bg-white/5 md:hidden"
            aria-label="Open menu"
          >
            {drawerOpen ? <X size={18} /> : <Menu size={18} />}
          </button>
        </div>

        {drawerOpen && (
          <div className="border-t border-[color:var(--color-border)] md:hidden">
            <div className="mx-auto max-w-6xl px-4 py-3 sm:px-6">
              <div className="flex flex-col gap-1">
                {primaryNav.map((item) => {
                  const active = isActiveSection(item.to, pathname);
                  return (
                    <Link
                      key={item.to}
                      to={item.to}
                      className={`rounded-md px-3 py-2 text-sm transition ${
                        active
                          ? "bg-[rgba(141,225,180,0.08)] text-[color:var(--color-accent)]"
                          : "text-[color:var(--color-fg-muted)] hover:bg-white/5"
                      }`}
                    >
                      {item.title}
                    </Link>
                  );
                })}
              </div>
              <div className="mt-3 border-t border-[color:var(--color-border)] pt-3">
                {nav.map((group) => (
                  <div key={group.title} className="mb-3">
                    <div className="mb-1 text-[10px] font-bold uppercase tracking-[0.16em] text-[color:var(--color-fg-subtle)]">
                      {group.title}
                    </div>
                    {group.items.map((leaf) => (
                      <NavLink
                        key={leaf.slug}
                        to={`/${leaf.slug}`}
                        className={({ isActive }) =>
                          `block rounded-md px-2 py-1.5 text-sm transition ${
                            isActive
                              ? "text-[color:var(--color-accent)]"
                              : "text-[color:var(--color-fg-muted)] hover:text-[color:var(--color-fg)]"
                          }`
                        }
                      >
                        {leaf.title}
                      </NavLink>
                    ))}
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
      </header>

      <SearchDialog open={searchOpen} onOpenChange={setSearchOpen} />
    </>
  );
}
