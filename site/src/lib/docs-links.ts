import { posix } from "node:path";
import type { Element, Root } from "hast";
import { visit } from "unist-util-visit";
import { docByFile } from "@/lib/docs-manifest";
import { REPO_URL } from "@/lib/links";

const EXTERNAL = /^([a-z][a-z0-9+.-]*:|#|\/\/)/i;

function resolveHref(href: string, file: string): string {
  const [target, hash = ""] = href.split("#");
  const resolved = posix.normalize(posix.join(posix.dirname(file), target));
  const doc = docByFile(resolved);
  const suffix = hash ? `#${hash}` : "";
  if (doc) return `/docs/${doc.slug}${suffix}`;
  return `${REPO_URL}/blob/main/${resolved}${suffix}`;
}

/** Points relative links in a markdown file at the matching docs page, or at the file on GitHub. */
export function rehypeDocLinks(options: { file: string }) {
  return (tree: Root) => {
    visit(tree, "element", (node: Element) => {
      const href = node.properties?.href;
      if (node.tagName === "a" && typeof href === "string" && !EXTERNAL.test(href)) {
        node.properties.href = resolveHref(href, options.file);
      }
    });
  };
}
