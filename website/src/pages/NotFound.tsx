import { Link } from "react-router-dom";

export default function NotFound() {
  return (
    <div className="mx-auto max-w-2xl px-6 py-16 text-center">
      <p className="text-xs font-bold uppercase tracking-[0.18em] text-[color:var(--color-fg-subtle)]">
        404
      </p>
      <h1 className="mt-3 font-serif text-4xl font-medium tracking-tight">Page not found</h1>
      <p className="mt-3 text-[color:var(--color-fg-muted)]">
        That page moved or never existed.
      </p>
      <Link
        to="/"
        className="mt-6 inline-flex rounded-full border border-[rgba(141,225,180,0.55)] px-5 py-2 text-sm font-semibold text-[color:var(--color-accent)] transition hover:bg-[rgba(141,225,180,0.12)]"
      >
        Back to home
      </Link>
    </div>
  );
}
