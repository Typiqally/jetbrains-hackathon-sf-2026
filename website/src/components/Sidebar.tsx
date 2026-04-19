import { NavLink } from "react-router-dom";
import { nav } from "../nav";

export default function Sidebar() {
  return (
    <aside className="hidden w-56 shrink-0 border-r border-[color:var(--color-border)] pr-6 lg:block">
      <div className="sticky top-16 max-h-[calc(100vh-5rem)] overflow-y-auto pb-10 pt-8">
        {nav.map((group) => (
          <div key={group.title} className="mb-6">
            <div className="mb-2 text-[10px] font-bold uppercase tracking-[0.18em] text-[color:var(--color-fg-subtle)]">
              {group.title}
            </div>
            <ul className="space-y-0.5">
              {group.items.map((leaf) => (
                <li key={leaf.slug}>
                  <NavLink
                    to={`/${leaf.slug}`}
                    end
                    className={({ isActive }) =>
                      `block rounded-md px-2 py-1 text-[13px] transition ${
                        isActive
                          ? "bg-[rgba(141,225,180,0.08)] text-[color:var(--color-accent)]"
                          : "text-[color:var(--color-fg-muted)] hover:text-[color:var(--color-fg)]"
                      }`
                    }
                  >
                    {leaf.title}
                  </NavLink>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
    </aside>
  );
}
