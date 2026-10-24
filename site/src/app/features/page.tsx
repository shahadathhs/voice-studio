import { Breadcrumbs } from "@/components/breadcrumbs";
import { FeatureGrid } from "@/components/feature-grid";
import { PageShell } from "@/components/page-shell";
import { PageTitle } from "@/components/page-title";
import { pageMetadata } from "@/lib/seo";

export const metadata = pageMetadata({
  title: "Features",
  description:
    "Transcribe, identify speakers, record with live captions, speak, make voices, search and summarise, and manage models, all offline.",
  path: "/features",
});

export default function FeaturesPage() {
  return (
    <PageShell>
      <div className="px-6 pt-8 md:px-10">
        <Breadcrumbs trail={[{ name: "Home", href: "/" }, { name: "Features" }]} />
      </div>
      <PageTitle eyebrow="Features" title="Seven screens. One engine. No cloud.">
        <p>
          Each screen has its own page with what it does, the matching command-line commands and its
          honest limits. Screenshots show the real interface with made-up sample data.
        </p>
      </PageTitle>
      <div className="border-t border-line">
        <FeatureGrid />
      </div>
    </PageShell>
  );
}
