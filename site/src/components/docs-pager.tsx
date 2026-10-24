import Link from "next/link";
import { DOCS } from "@/lib/docs-manifest";

/** Previous and next page, in the order of the sidebar. */
export function DocsPager({ slug }: { slug: string }) {
  const i = DOCS.findIndex((doc) => doc.slug === slug);
  const prev = i > 0 ? DOCS[i - 1] : undefined;
  const next = i >= 0 && i < DOCS.length - 1 ? DOCS[i + 1] : undefined;
  const base =
    "block flex-1 rounded-md border border-line p-4 transition-colors duration-150 hover:bg-raised";
  return (
    <div className="mt-12 flex gap-4">
      {prev ? (
        <Link href={`/docs/${prev.slug}`} className={base}>
          <span className="font-mono text-[11px] tracking-widest text-muted uppercase">
            ← Previous
          </span>
          <span className="mt-1 block font-medium">{prev.title}</span>
        </Link>
      ) : (
        <span className="flex-1" />
      )}
      {next ? (
        <Link href={`/docs/${next.slug}`} className={`${base} text-right`}>
          <span className="font-mono text-[11px] tracking-widest text-muted uppercase">Next →</span>
          <span className="mt-1 block font-medium">{next.title}</span>
        </Link>
      ) : (
        <span className="flex-1" />
      )}
    </div>
  );
}
