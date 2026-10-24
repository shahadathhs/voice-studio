import { FeatureGrid } from "@/components/feature-grid";
import { HomeHero } from "@/components/home-hero";
import { PageShell } from "@/components/page-shell";
import { PrivacyCallout } from "@/components/privacy-callout";
import { Screenshot } from "@/components/screenshot";

export default function HomePage() {
  return (
    <PageShell>
      <HomeHero />
      <div className="mt-12">
        <Screenshot
          name="transcript"
          alt="A transcript with three speakers, timestamps and a speaker panel"
        />
      </div>
      <FeatureGrid />
      <PrivacyCallout />
    </PageShell>
  );
}
