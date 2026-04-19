import { useEffect, useState } from "react";

export type TocItem = { id: string; text: string; level: number };

export default function Toc({ items }: { items: TocItem[] }) {
  const [active, setActive] = useState<string | null>(null);

  useEffect(() => {
    if (!items.length) return;
    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((e) => e.isIntersecting)
          .sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top);
        if (visible[0]?.target.id) setActive(visible[0].target.id);
      },
      { rootMargin: "-80px 0px -70% 0px", threshold: [0, 1] },
    );
    items.forEach((item) => {
      const el = document.getElementById(item.id);
      if (el) observer.observe(el);
    });
    return () => observer.disconnect();
  }, [items]);

  if (!items.length) return null;

  return (
    <aside className="hidden w-56 shrink-0 pl-6 xl:block">
      <div className="sticky top-16 max-h-[calc(100vh-5rem)] overflow-y-auto pb-10 pt-8">
        <div className="mb-2 text-[10px] font-bold uppercase tracking-[0.18em] text-[color:var(--color-fg-subtle)]">
          On this page
        </div>
        <ul className="space-y-1">
          {items.map((item) => (
            <li
              key={item.id}
              style={{ paddingLeft: `${(item.level - 2) * 0.8}rem` }}
            >
              <a
                href={`#${item.id}`}
                className={`block text-[12px] transition ${
                  active === item.id
                    ? "text-[color:var(--color-accent)]"
                    : "text-[color:var(--color-fg-muted)] hover:text-[color:var(--color-fg)]"
                }`}
              >
                {item.text}
              </a>
            </li>
          ))}
        </ul>
      </div>
    </aside>
  );
}
