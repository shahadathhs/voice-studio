import { REPO } from "@/lib/links";

export interface ReleaseAsset {
  name: string;
  url: string;
  size: number;
}

export interface LatestRelease {
  tag: string;
  url: string;
  assets: ReleaseAsset[];
}

interface ApiAsset {
  name: string;
  browser_download_url: string;
  size: number;
}

interface ApiRelease {
  tag_name: string;
  html_url: string;
  assets: ApiAsset[];
}

/** The newest published release, or null before the first one (or when GitHub is unreachable). */
export async function latestRelease(): Promise<LatestRelease | null> {
  try {
    const res = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
      headers: { Accept: "application/vnd.github+json" },
      next: { revalidate: 3600 },
    });
    if (!res.ok) return null;
    const data = (await res.json()) as ApiRelease;
    return {
      tag: data.tag_name,
      url: data.html_url,
      assets: data.assets
        .filter((a) => !a.name.endsWith(".sha256"))
        .map((a) => ({ name: a.name, url: a.browser_download_url, size: a.size })),
    };
  } catch {
    return null;
  }
}

export function formatSize(bytes: number): string {
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}
