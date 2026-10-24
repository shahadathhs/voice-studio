import { Breadcrumbs } from "@/components/breadcrumbs";
import { DocsShell } from "@/components/docs-shell";
import { ModelsTable } from "@/components/models-table";
import { KIND_LABELS, readCatalog } from "@/lib/models-data";
import { pageMetadata } from "@/lib/seo";

export const metadata = pageMetadata({
  title: "Models and licences",
  description:
    "Every model Voice Studio can download, with its size, languages and licence. Only permissive licences are listed.",
  path: "/docs/models",
});

export default function ModelsPage() {
  const catalog = readCatalog();
  return (
    <DocsShell current="models">
      <Breadcrumbs trail={[{ name: "Docs", href: "/docs" }, { name: "Models and licences" }]} />
      <h1 className="mt-4 text-3xl font-semibold tracking-tight sm:text-4xl">
        Models and licences
      </h1>
      <p className="mt-4 max-w-2xl text-muted">
        Models are not bundled: you download the ones you want, and each download is checked against
        a published SHA-256 checksum. Only permissive licences are listed. This page is read from
        the app&apos;s own catalog (<code className="font-mono">catalog/models.json</code>), so it
        matches what the app offers.
      </p>
      {Object.keys(KIND_LABELS).map((kind) => (
        <ModelsTable
          key={kind}
          kind={kind}
          models={catalog.filter((model) => model.kind === kind)}
        />
      ))}
    </DocsShell>
  );
}
