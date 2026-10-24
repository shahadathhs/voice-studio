import type { Metadata } from "next";
import { ButtonLink } from "@/components/button-link";
import { CodeBlock } from "@/components/code-block";
import { PageShell } from "@/components/page-shell";
import { CLI_EXAMPLES } from "@/lib/cli-examples";
import { CLI_DOCS_URL } from "@/lib/links";

export const metadata: Metadata = {
  title: "Command line",
  description: "The vtt command-line tool: the same engine as the app, for scripts and servers.",
};

export default function CliPage() {
  return (
    <PageShell>
      <h1 className="text-3xl font-semibold tracking-tight">Command line</h1>
      <p className="mt-3 max-w-2xl text-muted">
        The same engine as the app, for scripts and servers. Run{" "}
        <code className="font-mono">vtt --help</code> for every command.
      </p>
      <div className="mt-8">
        <CodeBlock code={CLI_EXAMPLES} />
      </div>
      <p className="mt-6 max-w-2xl text-sm text-muted">
        Whisper uses the GPU on macOS (Metal) automatically. On Windows and Linux it uses the CPU
        unless you build with the optional Vulkan or CUDA feature.
      </p>
      <div className="mt-6">
        <ButtonLink href={CLI_DOCS_URL}>CLI reference</ButtonLink>
      </div>
    </PageShell>
  );
}
