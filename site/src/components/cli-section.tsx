import { ButtonLink } from "@/components/button-link";
import { CodeBlock } from "@/components/code-block";
import { SectionFrame } from "@/components/section-frame";
import { CLI_EXAMPLES } from "@/lib/cli-examples";
import { CLI_DOCS_URL } from "@/lib/links";

export function CliSection({ index }: { index: string }) {
  return (
    <SectionFrame id="cli" index={index} name="Command line" tag="vtt">
      <div className="reveal grid gap-8 p-6 md:grid-cols-5 md:gap-10 md:p-10">
        <div className="md:col-span-2">
          <h2 className="text-2xl font-semibold tracking-tight sm:text-3xl">
            The same engine, for scripts.
          </h2>
          <p className="mt-4 text-muted">
            <code className="font-mono">vtt</code> does everything the app does: transcribe, record,
            speak, voices, summaries, translation and the library. Machine-readable output with{" "}
            <code className="font-mono">--json</code>, documented exit codes, shell completions.
          </p>
          <div className="mt-6">
            <ButtonLink href={CLI_DOCS_URL} quiet>
              CLI reference
            </ButtonLink>
          </div>
        </div>
        <div className="md:col-span-3">
          <CodeBlock code={CLI_EXAMPLES} />
        </div>
      </div>
    </SectionFrame>
  );
}
