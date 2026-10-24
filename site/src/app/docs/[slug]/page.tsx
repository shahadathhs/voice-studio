import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { Breadcrumbs } from "@/components/breadcrumbs";
import { DocsPager } from "@/components/docs-pager";
import { DocsShell } from "@/components/docs-shell";
import { DocsToc } from "@/components/docs-toc";
import { JsonLd } from "@/components/json-ld";
import { DOCS, docBySlug } from "@/lib/docs-manifest";
import { renderDoc } from "@/lib/docs-render";
import { readRepoFile, withoutTitle } from "@/lib/docs-source";
import { REPO_URL } from "@/lib/links";
import { pageMetadata } from "@/lib/seo";
import { breadcrumbs } from "@/lib/structured-data";

type Props = { params: Promise<{ slug: string }> };

/** Pages built from data (models, languages) have their own routes. */
const markdownDocs = DOCS.filter((doc) => doc.file);

export function generateStaticParams() {
  return markdownDocs.map((doc) => ({ slug: doc.slug }));
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const doc = docBySlug((await params).slug);
  if (!doc) return {};
  return pageMetadata({ title: doc.title, description: doc.summary, path: `/docs/${doc.slug}` });
}

export default async function DocPage({ params }: Props) {
  const doc = docBySlug((await params).slug);
  if (!doc?.file) notFound();
  const { html, headings } = await renderDoc(withoutTitle(readRepoFile(doc.file)), doc.file);
  return (
    <DocsShell current={doc.slug} toc={<DocsToc headings={headings} />}>
      <JsonLd
        data={breadcrumbs([
          { name: "Voice Studio", path: "/" },
          { name: "Documentation", path: "/docs" },
          { name: doc.title, path: `/docs/${doc.slug}` },
        ])}
      />
      <Breadcrumbs trail={[{ name: "Docs", href: "/docs" }, { name: doc.title }]} />
      <h1 className="mt-4 text-3xl font-semibold tracking-tight sm:text-4xl">{doc.title}</h1>
      <article className="docs-prose mt-6" dangerouslySetInnerHTML={{ __html: html }} />
      <p className="mt-10 text-sm text-muted">
        <a className="underline underline-offset-4" href={`${REPO_URL}/edit/main/${doc.file}`}>
          Edit this page on GitHub
        </a>
      </p>
      <DocsPager slug={doc.slug} />
    </DocsShell>
  );
}
