import { Band } from "@/components/band";
import { ButtonLink } from "@/components/button-link";
import { CliTeaser } from "@/components/cli-teaser";
import { FeatureGrid } from "@/components/feature-grid";
import { HomeHero } from "@/components/home-hero";
import { JsonLd } from "@/components/json-ld";
import { PageShell } from "@/components/page-shell";
import { EXTRAS } from "@/lib/extras";
import { SectionFrame } from "@/components/section-frame";
import { pageMetadata } from "@/lib/seo";
import { APP_SCHEMA } from "@/lib/structured-data";

export const metadata = pageMetadata({
  title: "Offline speech to text and text to speech",
  description:
    "Transcribe audio, label speakers, record with live captions, read text aloud and make your own voices on your own computer. Free, open source, no account.",
  path: "",
});

export default function HomePage() {
  return (
    <PageShell>
      <JsonLd data={APP_SCHEMA} />
      <HomeHero />
      <SectionFrame index="01" name="Features" tag="Seven screens">
        <FeatureGrid extras={EXTRAS} />
      </SectionFrame>
      <Band
        index="02"
        name="Private by design"
        tag="Privacy"
        title={
          <>
            Your audio stays <span className="text-accent">yours</span>.
          </>
        }
        action={
          <ButtonLink href="/privacy" quiet>
            How privacy works
          </ButtonLink>
        }
      >
        Audio and text never leave your computer. The only network use is downloading models, which
        you start yourself. No account, no telemetry.
      </Band>
      <CliTeaser index="03" />
      <Band
        index="04"
        name="Get Voice Studio"
        tag="Free"
        title={
          <>
            Build it <span className="text-accent">yourself</span>.
          </>
        }
        action={<ButtonLink href="/build">Build from source</ButtonLink>}
      >
        Voice Studio builds from source on macOS, Windows and Linux. A copy you build yourself
        starts normally, with no signing warnings.
      </Band>
    </PageShell>
  );
}
