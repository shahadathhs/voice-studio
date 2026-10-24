import { Breadcrumbs } from "@/components/breadcrumbs";
import { PageShell } from "@/components/page-shell";
import { PageTitle } from "@/components/page-title";
import { SectionFrame } from "@/components/section-frame";
import { REPO_URL } from "@/lib/links";
import { pageMetadata } from "@/lib/seo";

export const metadata = pageMetadata({
  title: "Privacy",
  description:
    "Voice Studio processes audio and text on your computer. No account, no telemetry. The only network use is downloading models you choose.",
  path: "/privacy",
});

const FACTS = [
  {
    title: "Nothing is uploaded",
    body: "Transcription, speaker labels, speech, voice cloning, summaries and translation all run on your computer. Your audio and text are never sent anywhere.",
  },
  {
    title: "Downloads you start",
    body: "Models are not bundled. The only network use is downloading a model, which you start, and every download is checked against a published SHA-256 checksum.",
  },
  {
    title: "No account, no telemetry",
    body: "There is nothing to sign in to, and nothing in the app reports back.",
  },
  {
    title: "Files stay where they are",
    body: "Source audio is read in place. A copy is made only if you turn on keep a copy. Recordings stay on this computer until you discard them.",
  },
  {
    title: "Cloned voices need consent",
    body: "Cloning asks you to confirm you own the voice or have permission. Cloned voices are marked, can be deleted, and saved audio carries a synthetic voice note.",
  },
  {
    title: "Open source",
    body: "MIT licensed, so you can read the code and check every claim on this page.",
  },
];

export default function PrivacyPage() {
  return (
    <PageShell>
      <div className="px-6 pt-8 md:px-10">
        <Breadcrumbs trail={[{ name: "Home", href: "/" }, { name: "Privacy" }]} />
      </div>
      <PageTitle
        eyebrow="Privacy"
        title={
          <>
            Your audio stays <span className="text-accent">yours</span>.
          </>
        }
      >
        <p>What Voice Studio does and does not do with your data.</p>
      </PageTitle>
      <SectionFrame index="01" name="What stays on your computer">
        <ul className="reveal-group grid gap-px bg-line sm:grid-cols-2 lg:grid-cols-3">
          {FACTS.map((fact) => (
            <li key={fact.title} className="reveal-item bg-bg p-6">
              <h2 className="font-medium">{fact.title}</h2>
              <p className="mt-2 text-sm text-muted">{fact.body}</p>
            </li>
          ))}
        </ul>
      </SectionFrame>
      <SectionFrame index="02" name="This website">
        <p className="reveal max-w-2xl px-6 py-10 text-muted md:px-10">
          This site is static and has no analytics. The source of the app and of this site is in{" "}
          <a className="underline underline-offset-4" href={REPO_URL}>
            the repository
          </a>
          .
        </p>
      </SectionFrame>
    </PageShell>
  );
}
