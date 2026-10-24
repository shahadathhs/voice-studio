import Link from "next/link";
import { DOCS, GROUPS } from "@/lib/docs-manifest";

/** The docs navigation: every page, grouped, with the current one marked. */
export function DocsSidebar({ current }: { current?: string }) {
  return (
    <nav aria-label="Documentation" className="space-y-6 text-sm">
      <Link
        href="/docs"
        className={`block font-mono text-[11px] tracking-widest uppercase ${current === undefined ? "text-accent" : "text-muted hover:text-ink"}`}
      >
        Overview
      </Link>
      {GROUPS.map((group) => (
        <div key={group}>
          <h2 className="font-mono text-[11px] tracking-widest text-muted uppercase">{group}</h2>
          <ul className="mt-2 space-y-1">
            {DOCS.filter((doc) => doc.group === group).map((doc) => (
              <li key={doc.slug}>
                <Link
                  href={`/docs/${doc.slug}`}
                  aria-current={doc.slug === current ? "page" : undefined}
                  className={`block border-l py-1 pl-3 transition-colors duration-150 ${doc.slug === current ? "border-accent text-ink" : "border-line text-muted hover:text-ink"}`}
                >
                  {doc.title}
                </Link>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </nav>
  );
}
