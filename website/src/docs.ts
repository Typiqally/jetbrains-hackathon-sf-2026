const raw = import.meta.glob("../../docs/**/*.md", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

export type DocEntry = {
  slug: string;
  title?: string;
  body: string;
};

const docs: Record<string, DocEntry> = {};

for (const [path, content] of Object.entries(raw)) {
  const slug = path
    .replace(/^.*\/docs\//, "")
    .replace(/\.md$/, "")
    .replace(/\/index$/, "");

  const { frontmatter, body } = splitFrontmatter(content);
  const title =
    frontmatter.title ??
    body.match(/^#\s+(.+)$/m)?.[1]?.trim() ??
    undefined;

  docs[slug] = { slug, title, body };
}

export function getDoc(slug: string): DocEntry | undefined {
  return docs[slug];
}

export function allDocs(): DocEntry[] {
  return Object.values(docs);
}

function splitFrontmatter(src: string): {
  frontmatter: Record<string, string>;
  body: string;
} {
  const match = src.match(/^---\n([\s\S]*?)\n---\n?([\s\S]*)$/);
  if (!match) return { frontmatter: {}, body: src };
  const frontmatter: Record<string, string> = {};
  for (const line of match[1].split("\n")) {
    const kv = line.match(/^([a-zA-Z0-9_-]+):\s*(.*)$/);
    if (kv) frontmatter[kv[1]] = kv[2].replace(/^["']|["']$/g, "");
  }
  return { frontmatter, body: match[2] };
}
