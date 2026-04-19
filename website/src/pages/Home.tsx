import { Link } from "react-router-dom";
import { CodeBlock } from "../components/CodeBlock";

const terminalSample = `$ lintropy check .

warning[no-unwrap]: avoid .unwrap() on \`client\`
  --> src/handlers/users.rs:42:18
  help: replace with \`client.expect("TODO: handle error")\`

error[api-only-in-src-api]: API handlers must live under src/api/
  --> src/features/users/create_user.rs:1:1

Summary: 1 error, 1 warning, 2 files affected.`;

const ruleSample = `severity: warning
message: "avoid .unwrap() on \`{{recv}}\`"
fix: '{{recv}}.expect("TODO: handle error")'
language: rust
query: |
  (call_expression
    function: (field_expression
      value: (_) @recv
      field: (field_identifier) @method)
    (#eq? @method "unwrap")) @match`;

const quickstart = `brew tap Typiqally/lintropy
brew install lintropy

lintropy init
lintropy check .`;

export default function Home() {
  return (
    <div className="mx-auto w-full max-w-6xl px-4 py-10 sm:px-6 sm:py-14">
      <section className="pt-4 pb-10">
        <p className="mb-3 text-[11px] font-bold uppercase tracking-[0.18em] text-[color:var(--color-fg-subtle)]">
          Repo-local linting
        </p>
        <h1
          className="font-serif font-medium tracking-[-0.04em] text-[color:var(--color-fg)]"
          style={{ fontSize: "clamp(2.5rem, 5.5vw, 4.4rem)", lineHeight: 0.96 }}
        >
          Lintropy
        </h1>
        <p
          className="mt-3 text-[color:var(--color-fg)]"
          style={{ fontSize: "clamp(1.02rem, 1.8vw, 1.22rem)" }}
        >
          The linter for rules your repo actually cares about.
        </p>
        <p className="mt-4 max-w-2xl text-[color:var(--color-fg-muted)]">
          Put architecture boundaries, banned APIs, migration policy, and agent guardrails in version
          control. One root config. Small YAML rule files. Structural matching for Rust.
        </p>

        <div className="mt-6 mb-5 flex flex-wrap gap-3">
          <Link
            to="/getting-started"
            className="inline-flex items-center rounded-full border border-[color:var(--color-accent)] bg-[color:var(--color-accent)] px-5 py-2 text-sm font-semibold text-[color:var(--color-accent-ink)] shadow-[0_10px_28px_rgba(141,225,180,0.18)] transition hover:bg-[color:var(--color-accent-hover)]"
          >
            Get started
          </Link>
          <a
            href="https://github.com/Typiqally/lintropy"
            className="inline-flex items-center rounded-full border border-[rgba(141,225,180,0.55)] px-5 py-2 text-sm font-semibold text-[color:var(--color-accent)] transition hover:bg-[rgba(141,225,180,0.12)]"
          >
            View source
          </a>
        </div>

        <ul className="max-w-3xl space-y-1 pl-5 text-[color:var(--color-fg-muted)] marker:text-[color:var(--color-fg-faint)]">
          <li>
            <strong className="text-[color:var(--color-fg)]">Repo-local:</strong> rules live with the
            code they govern.
          </li>
          <li>
            <strong className="text-[color:var(--color-fg)]">Structural:</strong> Rust rules use
            tree-sitter queries, not brittle grep.
          </li>
          <li>
            <strong className="text-[color:var(--color-fg)]">Practical:</strong> messages, severity,
            and autofix stay in one place.
          </li>
          <li>
            <strong className="text-[color:var(--color-fg)]">Agent-friendly:</strong> explicit enough
            for humans, simple enough for coding agents.
          </li>
        </ul>
      </section>

      <section
        aria-label="Lintropy examples"
        className="grid gap-4 md:grid-cols-[1.25fr_0.95fr]"
      >
        <Card>
          <Windowbar />
          <CodeBlock
            code={terminalSample}
            language="console"
            className="m-0 overflow-auto px-5 py-4 font-mono text-[13.5px] leading-[1.65] text-[color:var(--color-fg)]"
          />
        </Card>

        <Card>
          <p className="px-5 pt-4 text-[11px] font-bold uppercase tracking-[0.15em] text-[color:var(--color-fg-subtle)]">
            Example rule
          </p>
          <CodeBlock
            code={ruleSample}
            language="lintropy"
            className="m-0 overflow-auto px-5 py-4 font-mono text-[13px] leading-[1.6] text-[color:var(--color-fg)]"
          />
        </Card>
      </section>

      <section className="mt-12">
        <Row
          heading="Rules live in the repo."
          body="Review them like code. Change them with the codebase. Stop hiding project policy in wiki pages, onboarding calls, and reviewer lore."
        />
        <Row
          heading="Simple shape."
          body={
            <>
              A root <Code>lintropy.yaml</Code>. A <Code>.lintropy/</Code> folder. One rule per file,
              or grouped rules where that helps.
            </>
          }
        />
        <Row
          heading="Start."
          body={
            <CodeBlock
              code={quickstart}
              language="console"
              className="m-0 overflow-auto rounded-xl border border-[color:var(--color-border)] bg-[color:var(--color-surface)] px-4 py-3 font-mono text-[13px] leading-[1.6] text-[color:var(--color-fg)]"
            />
          }
        />
      </section>
    </div>
  );
}

function Card({ children }: { children: React.ReactNode }) {
  return (
    <div
      className="overflow-hidden rounded-[1.3rem] border border-white/10 shadow-[0_24px_60px_rgba(0,0,0,0.35)]"
      style={{ background: "linear-gradient(180deg, #141414 0%, #101010 100%)" }}
    >
      {children}
    </div>
  );
}

function Windowbar() {
  return (
    <div className="flex items-center gap-2 border-b border-white/10 px-4 py-3">
      <span className="h-2.5 w-2.5 rounded-full" style={{ background: "#f17373" }} />
      <span className="h-2.5 w-2.5 rounded-full" style={{ background: "#ddb15d" }} />
      <span className="h-2.5 w-2.5 rounded-full" style={{ background: "#76cb8b" }} />
    </div>
  );
}

function Row({
  heading,
  body,
}: {
  heading: string;
  body: React.ReactNode;
}) {
  return (
    <div className="grid gap-5 border-t border-[color:var(--color-border)] py-5 md:grid-cols-[0.9fr_1.1fr]">
      <h2
        className="m-0 font-serif font-medium tracking-[-0.02em] text-[color:var(--color-fg)]"
        style={{ fontSize: "clamp(1.3rem, 2vw, 1.8rem)", lineHeight: 1.06 }}
      >
        {heading}
      </h2>
      <div className="text-[color:var(--color-fg-muted)]">{body}</div>
    </div>
  );
}

function Code({ children }: { children: React.ReactNode }) {
  return (
    <code className="rounded-md border border-[color:var(--color-border)] bg-white/5 px-1.5 py-0.5 font-mono text-[0.86em]">
      {children}
    </code>
  );
}
