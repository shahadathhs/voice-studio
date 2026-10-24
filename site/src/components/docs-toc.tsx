import type { Heading } from "@/lib/docs-render";

/** "On this page": the headings of the current doc. */
export function DocsToc({ headings }: { headings: Heading[] }) {
  if (headings.length < 3) return null;
  return (
    <nav aria-label="On this page" className="text-sm">
      <h2 className="font-mono text-[11px] tracking-widest text-muted uppercase">On this page</h2>
      <ul className="mt-3 space-y-1.5">
        {headings.map((heading) => (
          <li key={heading.id} className={heading.level === 3 ? "pl-3" : ""}>
            <a
              href={`#${heading.id}`}
              className="text-muted transition-colors duration-150 hover:text-ink"
            >
              {heading.text}
            </a>
          </li>
        ))}
      </ul>
    </nav>
  );
}
