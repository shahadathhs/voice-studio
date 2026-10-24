import type { Metadata } from "next";
import { ButtonLink } from "@/components/button-link";
import { PageShell } from "@/components/page-shell";
import { LATEST_RELEASE_URL, RELEASES_URL } from "@/lib/links";
import { formatSize, latestRelease } from "@/lib/releases";

export const metadata: Metadata = {
  title: "Download",
  description: "Installers for macOS, Windows and Linux, and the vtt command-line tool.",
};

export const revalidate = 3600;

export default async function DownloadPage() {
  const release = await latestRelease();
  return (
    <PageShell>
      <h1 className="text-3xl font-semibold tracking-tight">Download</h1>
      {release ? (
        <>
          <p className="mt-3 text-muted">
            Latest release:{" "}
            <a className="underline underline-offset-4" href={release.url}>
              {release.tag}
            </a>
            . Each file has a matching <code className="font-mono">.sha256</code> on the release
            page.
          </p>
          <ul className="mt-8 divide-y divide-line rounded-md border border-line bg-surface">
            {release.assets.map((asset) => (
              <li
                key={asset.name}
                className="flex items-center justify-between gap-4 px-4 py-3 text-sm"
              >
                <a href={asset.url} className="break-all hover:underline">
                  {asset.name}
                </a>
                <span className="shrink-0 text-muted">{formatSize(asset.size)}</span>
              </li>
            ))}
          </ul>
        </>
      ) : (
        <>
          <p className="mt-3 max-w-2xl text-muted">
            The first release is not published yet. Until then, build from source (see the README).
          </p>
          <div className="mt-6 flex items-center gap-5">
            <ButtonLink href={RELEASES_URL}>Releases on GitHub</ButtonLink>
          </div>
        </>
      )}
      <p className="mt-8 max-w-2xl text-sm text-muted">
        Builds are not signed yet. macOS and Windows will warn the first time you open the app: on
        macOS right-click the app and choose Open, on Windows choose More info, then Run anyway.{" "}
        <a className="underline underline-offset-4" href={LATEST_RELEASE_URL}>
          All files on GitHub
        </a>
        .
      </p>
    </PageShell>
  );
}
