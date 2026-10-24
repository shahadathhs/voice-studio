import type { Metadata } from "next";
import { PageShell } from "@/components/page-shell";

export const metadata: Metadata = {
  title: "Privacy",
  description: "Audio and text never leave your computer. No account, no telemetry.",
};

const POINTS = [
  "Audio and text are processed on your computer and are never uploaded.",
  "The only network use is downloading models, which you start yourself. Each download is checked against a published checksum.",
  "There is no account, no telemetry and no analytics in the app.",
  "The source is open (MIT), so you can check all of this.",
];

export default function PrivacyPage() {
  return (
    <PageShell>
      <h1 className="text-3xl font-semibold tracking-tight">Privacy</h1>
      <ul className="mt-8 max-w-2xl list-disc space-y-3 pl-5 text-muted">
        {POINTS.map((point) => (
          <li key={point}>{point}</li>
        ))}
      </ul>
    </PageShell>
  );
}
