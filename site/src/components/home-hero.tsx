import { ButtonLink } from "@/components/button-link";
import { Screenshot } from "@/components/screenshot";
import { REPO_URL } from "@/lib/links";

export function HomeHero() {
  return (
    <section>
      <div className="hero-in px-6 pt-16 pb-12 text-center">
        <p className="font-mono text-[11px] tracking-widest text-muted uppercase">
          Windows · macOS · Linux · open source
        </p>
        <h1 className="mx-auto mt-5 max-w-3xl text-4xl font-semibold tracking-tight sm:text-6xl">
          Speech to text and back, <span className="text-accent">offline</span>.
        </h1>
        <p className="mx-auto mt-6 max-w-2xl text-lg text-muted">
          Transcribe recordings, label who said what, record with live captions, read text aloud and
          make your own voices, all on your own computer. No account, no cloud, no per-minute fees.
        </p>
        <div className="mt-8 flex flex-wrap items-center justify-center gap-3">
          <ButtonLink href="/build">Build from source</ButtonLink>
          <ButtonLink href="/features" quiet>
            See the features
          </ButtonLink>
          <ButtonLink href={REPO_URL} quiet>
            GitHub
          </ButtonLink>
        </div>
      </div>
      <div className="border-t border-line bg-sunken p-4 sm:p-8">
        <div className="hero-shot mx-auto max-w-5xl overflow-hidden rounded-md border border-line-strong">
          <Screenshot
            name="transcript"
            alt="A transcript with three speakers, timestamps, a waveform and a speaker panel"
            width={2560}
            height={1600}
            priority
          />
        </div>
        <p className="mt-3 text-center font-mono text-[11px] tracking-widest text-muted uppercase">
          The real interface, with made-up sample data
        </p>
      </div>
    </section>
  );
}
