import type { Metadata } from "next";
import { BuildGuide } from "@/components/build-guide";
import { PageShell } from "@/components/page-shell";
import { REPO_URL } from "@/lib/links";

export const metadata: Metadata = {
  title: "Build from source",
  description:
    "Build Voice Studio and the vtt command-line tool from source on macOS, Windows or Linux.",
};

export default function BuildPage() {
  return (
    <PageShell>
      <div className="min-h-[60vh] px-6 py-12 md:px-10">
        <h1 className="text-3xl font-semibold tracking-tight">Build from source</h1>
        <p className="mt-3 max-w-2xl text-muted">
          There are no prebuilt downloads: installers need code signing, and unsigned ones are
          blocked or warned about by macOS and Windows. A copy you build yourself starts normally.
        </p>
        <BuildGuide />
        <p className="mt-10 max-w-2xl text-sm text-muted">
          GPU on Windows and Linux, and the full reference, are in{" "}
          <a
            className="underline underline-offset-4"
            href={`${REPO_URL}/blob/main/docs/building.md`}
          >
            docs/building.md
          </a>
          .
        </p>
      </div>
    </PageShell>
  );
}
