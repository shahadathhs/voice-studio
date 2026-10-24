import { readFileSync } from "node:fs";
import { join } from "node:path";

/** The repository root: the site's folder is `site/`, one level down. */
export const REPO_ROOT = join(/* turbopackIgnore: true */ process.cwd(), "..");

/** Reads a file of the repository at build time. The pages are all prerendered, so the build does not
 *  need to trace these reads into the output. */
export function readRepoFile(path: string): string {
  return readFileSync(join(/* turbopackIgnore: true */ REPO_ROOT, path), "utf8");
}

/** The markdown without its first heading, which the page shows as its own title. */
export function withoutTitle(markdown: string): string {
  return markdown.replace(/^# .*\n+/, "");
}
