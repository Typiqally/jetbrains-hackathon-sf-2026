export type NavLeaf = { title: string; slug: string };
export type NavGroup = { title: string; items: NavLeaf[] };

/**
 * Slug maps to docs/<slug>.md (or docs/<slug>/index.md for section roots
 * without trailing segments — handled by loader). Slugs are also the URL
 * path beneath the site base (e.g., slug "overview" -> /overview).
 */
export const nav: NavGroup[] = [
  {
    title: "Start here",
    items: [
      { title: "Overview", slug: "overview" },
      { title: "Getting started", slug: "getting-started" },
    ],
  },
  {
    title: "Reference",
    items: [
      { title: "Configuration", slug: "configuration" },
      { title: "Rule language", slug: "rule-language" },
      { title: "CLI guide", slug: "cli" },
    ],
  },
  {
    title: "Integrations",
    items: [
      { title: "Overview", slug: "integrations" },
      { title: "VS Code & Cursor", slug: "integrations/vscode" },
      { title: "JetBrains IDEs", slug: "integrations/jetbrains" },
      { title: "Claude Code", slug: "integrations/claude-code" },
      { title: "Other LSP editors", slug: "integrations/other-editors" },
      { title: "Other agents", slug: "integrations/other-agents" },
    ],
  },
  {
    title: "Support",
    items: [{ title: "Troubleshooting", slug: "troubleshooting" }],
  },
];

export const primaryNav: { title: string; to: string }[] = [
  { title: "Home", to: "/" },
  { title: "Start here", to: "/overview" },
  { title: "Reference", to: "/configuration" },
  { title: "Integrations", to: "/integrations" },
  { title: "Support", to: "/troubleshooting" },
];

export function findLeaf(slug: string): { group: NavGroup; leaf: NavLeaf } | null {
  for (const group of nav) {
    for (const leaf of group.items) {
      if (leaf.slug === slug) return { group, leaf };
    }
  }
  return null;
}

export function neighbors(slug: string): { prev?: NavLeaf; next?: NavLeaf } {
  const flat = nav.flatMap((g) => g.items);
  const idx = flat.findIndex((l) => l.slug === slug);
  if (idx < 0) return {};
  return { prev: flat[idx - 1], next: flat[idx + 1] };
}
