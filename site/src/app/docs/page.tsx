import Link from "next/link";
import { DocsShell } from "@/components/docs-shell";
import { DOCS, GROUPS } from "@/lib/docs-manifest";
import { pageMetadata } from "@/lib/seo";

export const metadata = pageMetadata({
  title: "Documentation",
  description:
    "Documentation for Voice Studio: build from source, features, the command-line reference, models and licences, languages, architecture and how to contribute.",
  path: "/docs",
});

export default function DocsIndexPage() {
  return (
    <DocsShell>
      <p className="font-mono text-[11px] tracking-widest text-muted uppercase">Documentation</p>
      <h1 className="mt-3 text-3xl font-semibold tracking-tight sm:text-4xl">
        Everything about Voice Studio.
      </h1>
      <p className="mt-4 max-w-2xl text-lg text-muted">
        These pages are the project&apos;s own documentation, rendered from the files in the
        repository.
      </p>
      {GROUPS.map((group) => (
        <section key={group} className="mt-10">
          <h2 className="font-mono text-[11px] tracking-widest text-muted uppercase">{group}</h2>
          <ul className="mt-3 grid gap-px overflow-hidden rounded-md border border-line bg-line sm:grid-cols-2">
            {DOCS.filter((doc) => doc.group === group).map((doc) => (
              <li key={doc.slug} className="bg-bg">
                <Link
                  href={`/docs/${doc.slug}`}
                  className="block h-full p-5 transition-colors duration-150 hover:bg-raised"
                >
                  <p className="font-medium">{doc.title}</p>
                  <p className="mt-1 text-sm text-muted">{doc.summary}</p>
                </Link>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </DocsShell>
  );
}
