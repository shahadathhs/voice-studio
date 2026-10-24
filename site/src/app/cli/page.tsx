import { Breadcrumbs } from "@/components/breadcrumbs";
import { ButtonLink } from "@/components/button-link";
import { CodeBlock } from "@/components/code-block";
import { PageShell } from "@/components/page-shell";
import { PageTitle } from "@/components/page-title";
import { SectionFrame } from "@/components/section-frame";
import { CLI_GROUPS, EXIT_CODES } from "@/lib/cli-groups";
import { CLI_DOCS_URL } from "@/lib/links";
import { pageMetadata } from "@/lib/seo";

export const metadata = pageMetadata({
  title: "Command line",
  description:
    "vtt is the Voice Studio command-line tool: transcribe, record, speak, voices, summaries and the library, with JSON output and documented exit codes.",
  path: "/cli",
});

export default function CliPage() {
  return (
    <PageShell>
      <div className="px-6 pt-8 md:px-10">
        <Breadcrumbs trail={[{ name: "Home", href: "/" }, { name: "Command line" }]} />
      </div>
      <PageTitle eyebrow="vtt" title="The same engine, for scripts and servers.">
        <p>
          <code className="font-mono">vtt</code> does what the app does. Run{" "}
          <code className="font-mono">vtt --help</code> for every command.
        </p>
      </PageTitle>
      {CLI_GROUPS.map((group, i) => (
        <SectionFrame key={group.title} index={String(i + 1).padStart(2, "0")} name={group.title}>
          <div className="reveal px-6 py-8 md:px-10">
            <CodeBlock code={group.code} />
          </div>
        </SectionFrame>
      ))}
      <SectionFrame index="06" name="Exit codes">
        <div className="reveal px-6 py-8 md:px-10">
          <dl className="grid max-w-2xl grid-cols-[4rem_1fr] gap-y-2 text-sm">
            {EXIT_CODES.map(([code, meaning]) => (
              <div key={code} className="contents">
                <dt className="font-mono text-accent">{code}</dt>
                <dd className="text-muted">{meaning}</dd>
              </div>
            ))}
          </dl>
          <div className="mt-8">
            <ButtonLink href={CLI_DOCS_URL} quiet>
              Full CLI reference
            </ButtonLink>
          </div>
        </div>
      </SectionFrame>
    </PageShell>
  );
}
