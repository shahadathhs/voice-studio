import { ButtonLink } from "@/components/button-link";
import { CodeBlock } from "@/components/code-block";
import { SectionFrame } from "@/components/section-frame";
import { CLI_GROUPS } from "@/lib/cli-groups";

export function CliTeaser({ index }: { index: string }) {
  return (
    <SectionFrame index={index} name="Command line" tag="vtt">
      <div className="reveal grid gap-8 px-6 py-14 md:grid-cols-5 md:gap-10 md:px-10">
        <div className="min-w-0 md:col-span-2">
          <h2 className="text-2xl font-semibold tracking-tight sm:text-4xl">
            The same engine, for scripts.
          </h2>
          <p className="mt-4 text-muted">
            <code className="font-mono">vtt</code> does what the app does: transcribe, record,
            speak, voices, summaries, translation and the library.
          </p>
          <div className="mt-7">
            <ButtonLink href="/cli" quiet>
              Command-line guide
            </ButtonLink>
          </div>
        </div>
        <div className="min-w-0 md:col-span-3">
          <CodeBlock code={CLI_GROUPS[0].code} />
        </div>
      </div>
    </SectionFrame>
  );
}
