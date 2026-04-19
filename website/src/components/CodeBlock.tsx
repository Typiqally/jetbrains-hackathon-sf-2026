import {
  Children,
  isValidElement,
  type ComponentPropsWithoutRef,
  type ReactNode,
} from "react";

type Token = {
  text: string;
  className?: string;
};

type CodeBlockProps = {
  code: string;
  language?: string;
  className?: string;
  codeClassName?: string;
};

type MarkdownCodeProps = ComponentPropsWithoutRef<"code"> & {
  inline?: boolean;
  children?: ReactNode;
};

type MarkdownPreProps = ComponentPropsWithoutRef<"pre"> & {
  children?: ReactNode;
};

const CONSOLE_LANGS = new Set(["bash", "console", "shell", "sh", "text", "zsh"]);
const QUERY_LANGS = new Set([
  "query",
  "s-lang",
  "scheme",
  "scm",
  "sexpr",
  "tree-sitter-query",
]);
const LINTROPY_LANGS = new Set(["lintropy", "yaml", "yml"]);
const VALUE_KEYWORDS = new Set(["error", "false", "info", "null", "rust", "true", "warning"]);

export function CodeBlock({
  code,
  language,
  className = "",
  codeClassName = "",
}: CodeBlockProps) {
  const resolvedLanguage = resolveLanguage(language, code);
  const lines = tokenize(code, resolvedLanguage);

  return (
    <pre className={className}>
      <code className={`code-block ${codeClassName}`.trim()}>
        {lines.map((line, index) => (
          <span className="code-line" key={index}>
            {line.map((token, tokenIndex) =>
              token.className ? (
                <span className={token.className} key={tokenIndex}>
                  {token.text}
                </span>
              ) : (
                <span key={tokenIndex}>{token.text}</span>
              ),
            )}
          </span>
        ))}
      </code>
    </pre>
  );
}

export function MarkdownCode({ inline, className, children, ...props }: MarkdownCodeProps) {
  return (
    <code className={className} {...props}>
      {children}
    </code>
  );
}

export function MarkdownPre({ children, ...props }: MarkdownPreProps) {
  const child = Children.toArray(children)[0];

  if (isValidElement(child)) {
    const childProps = child.props as { children?: ReactNode; className?: string };
    const code = flattenChildren(childProps.children);
    const language = childProps.className?.match(/language-([a-zA-Z0-9_-]+)/)?.[1];

    if (code !== null) {
      return (
        <CodeBlock
          code={code.replace(/\n$/, "")}
          language={language}
          className="m-0 overflow-auto rounded-[0.9rem] border border-[color:var(--color-border)] bg-[color:var(--color-surface)] px-4 py-3 text-[0.84rem] leading-[1.65]"
          codeClassName={childProps.className}
        />
      );
    }
  }

  return <pre {...props}>{children}</pre>;
}

function flattenChildren(children: ReactNode): string | null {
  const parts = Children.toArray(children).map((child) => {
    if (typeof child === "string" || typeof child === "number") return String(child);
    return null;
  });

  return parts.every((part) => part !== null) ? parts.join("") : null;
}

function resolveLanguage(language: string | undefined, code: string): "console" | "lintropy" | "query" | "plain" {
  const normalized = language?.toLowerCase();
  if (normalized && CONSOLE_LANGS.has(normalized)) return "console";
  if (normalized && QUERY_LANGS.has(normalized)) return "query";
  if (normalized && LINTROPY_LANGS.has(normalized)) {
    return looksLikeLintropyRule(code) ? "lintropy" : "plain";
  }

  if (looksLikeLintropyRule(code)) return "lintropy";
  if (looksLikeQuery(code)) return "query";
  if (/^\s*(\$|>)\s+/m.test(code)) return "console";
  return "plain";
}

function looksLikeLintropyRule(code: string): boolean {
  return /\b(query|severity|message|language|fix)\s*:/m.test(code);
}

function looksLikeQuery(code: string): boolean {
  return /@\w+|#(?:not-)?has-[a-z-]+\?|\([a-z_][a-z0-9_-]*/i.test(code);
}

function tokenize(code: string, language: "console" | "lintropy" | "query" | "plain"): Token[][] {
  switch (language) {
    case "console":
      return code.split("\n").map(tokenizeConsoleLine);
    case "lintropy":
      return tokenizeLintropy(code);
    case "query":
      return code.split("\n").map(tokenizeQueryLine);
    default:
      return code.split("\n").map((line) => [{ text: line }]);
  }
}

function tokenizeConsoleLine(line: string): Token[] {
  if (!line) return [{ text: "" }];

  const promptMatch = line.match(/^(\s*)([$>])(\s+)(.*)$/);
  if (promptMatch) {
    return [
      { text: promptMatch[1] },
      { text: promptMatch[2], className: "tok-prompt" },
      { text: promptMatch[3] },
      ...tokenizeCommand(promptMatch[4]),
    ];
  }

  const severityMatch = line.match(/^(\s*)(warning|error|help)(\[[^\]]+\])?(:\s*)(.*)$/);
  if (severityMatch) {
    return [
      { text: severityMatch[1] },
      { text: severityMatch[2], className: `tok-${severityMatch[2]}` },
      ...(severityMatch[3] ? [{ text: severityMatch[3], className: "tok-label" }] : []),
      { text: severityMatch[4], className: "tok-punctuation" },
      ...tokenizeInlineFragments(severityMatch[5]),
    ];
  }

  if (/^\s*--> /.test(line)) {
    const arrowMatch = line.match(/^(\s*-->\s+)(.*)$/);
    if (!arrowMatch) return tokenizeInlineFragments(line);
    return [
      { text: arrowMatch[1], className: "tok-gutter" },
      { text: arrowMatch[2], className: "tok-path" },
    ];
  }

  if (/^\s*Summary:/.test(line)) {
    const summaryMatch = line.match(/^(\s*Summary:)(\s*)(.*)$/);
    if (!summaryMatch) return tokenizeInlineFragments(line);
    return [
      { text: summaryMatch[1], className: "tok-key" },
      { text: summaryMatch[2] },
      ...tokenizeInlineFragments(summaryMatch[3]),
    ];
  }

  return tokenizeInlineFragments(line);
}

function tokenizeCommand(line: string): Token[] {
  const parts = line.split(/(\s+)/);
  let sawCommand = false;

  return parts.map((part) => {
    if (!part) return { text: part };
    if (/^\s+$/.test(part)) return { text: part };
    if (!sawCommand) {
      sawCommand = true;
      return { text: part, className: "tok-command" };
    }
    if (/^--?[a-z0-9][\w-]*$/i.test(part)) return { text: part, className: "tok-flag" };
    if (/^[./~]/.test(part)) return { text: part, className: "tok-path" };
    return classifyInlineFragment(part);
  });
}

function tokenizeInlineFragments(text: string): Token[] {
  return tokenizeWithRegex(
    text,
    /`[^`]+`|"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|(?:\{\{[^}]+\}\})|(?:\b\d+(?:\.\d+)?\b)|(?:[./~][^\s]*)/g,
    classifyInlineFragment,
  );
}

function classifyInlineFragment(part: string): Token {
  if (/^`[^`]+`$/.test(part)) return { text: part, className: "tok-inline-code" };
  if (/^(?:".*"|'.*')$/.test(part)) return { text: part, className: "tok-string" };
  if (/^\{\{[^}]+\}\}$/.test(part)) return { text: part, className: "tok-interpolation" };
  if (/^\d+(?:\.\d+)?$/.test(part)) return { text: part, className: "tok-number" };
  if (/^[./~]/.test(part)) return { text: part, className: "tok-path" };
  return { text: part };
}

function tokenizeLintropy(code: string): Token[][] {
  const lines = code.split("\n");
  const result: Token[][] = [];
  let queryIndent: number | null = null;

  for (const line of lines) {
    const indent = line.match(/^\s*/)?.[0] ?? "";
    const trimmed = line.slice(indent.length);

    if (queryIndent !== null) {
      if (!trimmed) {
        result.push([{ text: line }]);
        continue;
      }
      if (indent.length > queryIndent) {
        result.push([{ text: indent }, ...tokenizeQueryLine(trimmed)]);
        continue;
      }
      queryIndent = null;
    }

    if (!trimmed) {
      result.push([{ text: "" }]);
      continue;
    }

    if (trimmed.startsWith("#")) {
      result.push([{ text: indent }, { text: trimmed, className: "tok-comment" }]);
      continue;
    }

    const fieldMatch = trimmed.match(/^([a-zA-Z_][\w-]*)(\s*:\s*)(.*)$/);
    if (fieldMatch) {
      const [, key, separator, rest] = fieldMatch;
      const lineTokens: Token[] = [
        { text: indent },
        { text: key, className: "tok-key" },
        { text: separator, className: "tok-punctuation" },
      ];

      if (rest) {
        if (key === "query" && rest.trim() === "|") {
          lineTokens.push({ text: rest, className: "tok-operator" });
          queryIndent = indent.length;
        } else {
          lineTokens.push(...tokenizeYamlValue(rest));
        }
      }

      result.push(lineTokens);
      continue;
    }

    result.push(tokenizeYamlValue(line));
  }

  return result;
}

function tokenizeYamlValue(text: string): Token[] {
  return tokenizeWithRegex(
    text,
    /\{\{[^}]+\}\}|"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|\b(?:true|false|null|warning|error|info|rust)\b|\b\d+(?:\.\d+)?\b|[|:[\]-]/g,
    (part) => {
      if (/^\{\{[^}]+\}\}$/.test(part)) return { text: part, className: "tok-interpolation" };
      if (/^(?:".*"|'.*')$/.test(part)) return { text: part, className: "tok-string" };
      if (VALUE_KEYWORDS.has(part)) return { text: part, className: "tok-keyword" };
      if (/^\d+(?:\.\d+)?$/.test(part)) return { text: part, className: "tok-number" };
      return { text: part, className: "tok-punctuation" };
    },
  );
}

function tokenizeQueryLine(line: string): Token[] {
  return tokenizeWithRegex(
    line,
    /"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|@[a-zA-Z_][\w-]*|#[a-zA-Z0-9_?!-]+|\b[a-zA-Z_][\w-]*(?=\s*:)|\b[a-zA-Z_][\w-]*[!?]?\b|[()|:]/g,
    (part, index, text) => {
      if (/^(?:".*"|'.*')$/.test(part)) return { text: part, className: "tok-string" };
      if (part.startsWith("@")) return { text: part, className: "tok-capture" };
      if (part.startsWith("#")) return { text: part, className: "tok-predicate" };
      if (/^[()|:]$/.test(part)) return { text: part, className: "tok-punctuation" };
      const next = text.slice(index + part.length);
      if (/^\s*:/.test(next)) return { text: part, className: "tok-field" };
      return { text: part, className: "tok-node" };
    },
  );
}

function tokenizeWithRegex(
  text: string,
  regex: RegExp,
  classify: (part: string, index: number, text: string) => Token,
): Token[] {
  const tokens: Token[] = [];
  let lastIndex = 0;

  for (const match of text.matchAll(regex)) {
    const value = match[0];
    const index = match.index ?? 0;
    if (index > lastIndex) tokens.push({ text: text.slice(lastIndex, index) });
    tokens.push(classify(value, index, text));
    lastIndex = index + value.length;
  }

  if (lastIndex < text.length) tokens.push({ text: text.slice(lastIndex) });
  return tokens.length ? tokens : [{ text }];
}
