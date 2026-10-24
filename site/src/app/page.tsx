import { CliSection } from "@/components/cli-section";
import { GetSection } from "@/components/get-section";
import { Hero } from "@/components/hero";
import { PageShell } from "@/components/page-shell";
import { PrivateSection } from "@/components/private-section";
import { ScreenSection } from "@/components/screen-section";
import { SCREENS } from "@/lib/screens";

const pad = (n: number) => String(n).padStart(2, "0");

export default function HomePage() {
  const after = SCREENS.length + 1;
  return (
    <PageShell>
      <Hero />
      <div id="screens" />
      {SCREENS.map((screen, i) => (
        <ScreenSection key={screen.id} screen={screen} index={pad(i + 1)} flip={i % 2 === 1} />
      ))}
      <PrivateSection index={pad(after)} />
      <CliSection index={pad(after + 1)} />
      <GetSection index={pad(after + 2)} />
    </PageShell>
  );
}
