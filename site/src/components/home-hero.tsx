import { ButtonLink } from "@/components/button-link";
import { REPO_URL } from "@/lib/links";

export function HomeHero() {
  return (
    <section className="max-w-2xl">
      <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl">
        Speech to text, and text to speech, without sending anything anywhere.
      </h1>
      <p className="mt-5 text-lg text-muted">
        Voice Studio transcribes recordings, labels who said what, and reads text aloud, all on your
        own computer. No account, no cloud, no per-minute fees. Models download once; after that it
        works offline.
      </p>
      <div className="mt-8 flex flex-wrap items-center gap-5">
        <ButtonLink href="/download">Download</ButtonLink>
        <ButtonLink href={REPO_URL} quiet>
          View on GitHub
        </ButtonLink>
      </div>
      <p className="mt-4 text-sm text-muted">Windows, macOS and Linux. Open source (MIT).</p>
    </section>
  );
}
