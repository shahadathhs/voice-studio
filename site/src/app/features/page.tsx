import type { Metadata } from "next";
import { PageShell } from "@/components/page-shell";
import { Screenshot } from "@/components/screenshot";

export const metadata: Metadata = {
  title: "Features",
  description: "Transcribe, label speakers, search, export and read aloud, all offline.",
};

const SCREENS = [
  {
    name: "transcribe",
    title: "Transcribe",
    note: "A queue for many files.",
    alt: "The transcription queue and recent transcripts",
  },
  {
    name: "models",
    title: "Models",
    note: "Grouped by what they are for.",
    alt: "Models grouped by purpose, with sizes and licences",
  },
  {
    name: "speak",
    title: "Speak",
    note: "Text to speech with six voices.",
    alt: "Text to speech with a voice list and speed control",
  },
  {
    name: "transcript",
    title: "Transcript",
    note: "Speakers, timestamps, playback and export.",
    alt: "A transcript with speakers and timestamps",
  },
];

export default function FeaturesPage() {
  return (
    <PageShell>
      <h1 className="text-3xl font-semibold tracking-tight">Features</h1>
      <p className="mt-3 max-w-2xl text-muted">
        Screenshots show the real interface with made-up sample data.
      </p>
      <ul className="mt-10 grid gap-10 md:grid-cols-2">
        {SCREENS.map((screen) => (
          <li key={screen.name}>
            <Screenshot name={screen.name} alt={screen.alt} />
            <h2 className="mt-3 font-medium">{screen.title}</h2>
            <p className="text-sm text-muted">{screen.note}</p>
          </li>
        ))}
      </ul>
    </PageShell>
  );
}
