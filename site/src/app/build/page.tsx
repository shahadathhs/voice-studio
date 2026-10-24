import { Breadcrumbs } from "@/components/breadcrumbs";
import { BuildGuide } from "@/components/build-guide";
import { PageShell } from "@/components/page-shell";
import { PageTitle } from "@/components/page-title";
import { REPO_URL } from "@/lib/links";
import { pageMetadata } from "@/lib/seo";

export const metadata = pageMetadata({
  title: "Build from source",
  description:
    "Build Voice Studio and the vtt command-line tool from source on macOS, Windows or Linux. A copy you build yourself starts without signing warnings.",
  path: "/build",
});

export default function BuildPage() {
  return (
    <PageShell>
      <div className="px-6 pt-8 md:px-10">
        <Breadcrumbs trail={[{ name: "Home", href: "/" }, { name: "Build from source" }]} />
      </div>
      <PageTitle eyebrow="Get it" title="Build from source.">
        <p>
          There are no prebuilt downloads: installers need code signing, and unsigned ones are
          blocked or warned about by macOS and Windows. A copy you build yourself starts normally.
        </p>
      </PageTitle>
      <div className="px-6 pb-12 md:px-10">
        <BuildGuide />
        <p className="mt-10 max-w-2xl text-sm text-muted">
          GPU on Windows and Linux, and the full reference, are in{" "}
          <a
            className="underline underline-offset-4"
            href={`${REPO_URL}/blob/main/docs/building.md`}
          >
            docs/building.md
          </a>
          .
        </p>
      </div>
    </PageShell>
  );
}
