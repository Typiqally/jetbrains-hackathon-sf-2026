import { Outlet, useLocation } from "react-router-dom";
import Header from "./components/Header";

export default function App() {
  const { pathname } = useLocation();
  const isHome = pathname === "/";

  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className={isHome ? "" : "flex-1"}>
        <Outlet />
      </main>
      <Footer />
    </div>
  );
}

function Footer() {
  return (
    <footer className="border-t border-[color:var(--color-border)] px-6 py-8 text-xs text-[color:var(--color-fg-faint)]">
      <div className="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-3">
        <span>
          Lintropy ·{" "}
          <a
            href="https://github.com/Typiqally/lintropy"
            className="underline-offset-4 hover:text-[color:var(--color-accent)] hover:underline"
          >
            GitHub
          </a>
        </span>
        <span>Repo-local linting.</span>
      </div>
    </footer>
  );
}
