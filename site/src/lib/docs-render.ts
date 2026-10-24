import type { Element, Root } from "hast";
import rehypeHighlight from "rehype-highlight";
import rehypeSlug from "rehype-slug";
import rehypeStringify from "rehype-stringify";
import remarkGfm from "remark-gfm";
import remarkParse from "remark-parse";
import remarkRehype from "remark-rehype";
import { unified } from "unified";
import { visit } from "unist-util-visit";
import { rehypeDocLinks } from "@/lib/docs-links";

export interface Heading {
  id: string;
  text: string;
  level: 2 | 3;
}

export interface RenderedDoc {
  html: string;
  headings: Heading[];
}

function textOf(node: Element): string {
  return node.children
    .map((child) =>
      child.type === "text" ? child.value : child.type === "element" ? textOf(child) : "",
    )
    .join("");
}

/** Wraps tables so wide ones scroll sideways instead of widening the page. */
function wrapTables() {
  return (tree: Root) => {
    visit(tree, "element", (node: Element, index, parent) => {
      if (node.tagName !== "table" || index === undefined || !parent) return;
      parent.children[index] = {
        type: "element",
        tagName: "div",
        properties: { className: ["table-wrap"] },
        children: [node],
      };
      return "skip";
    });
  };
}

/** Renders a markdown file to HTML and collects its h2/h3 headings for the table of contents. */
export async function renderDoc(markdown: string, file: string): Promise<RenderedDoc> {
  const headings: Heading[] = [];
  const collect = () => (tree: Root) => {
    visit(tree, "element", (node: Element) => {
      const id = node.properties?.id;
      if ((node.tagName === "h2" || node.tagName === "h3") && typeof id === "string") {
        headings.push({ id, text: textOf(node), level: node.tagName === "h2" ? 2 : 3 });
      }
    });
  };
  const html = await unified()
    .use(remarkParse)
    .use(remarkGfm)
    .use(remarkRehype)
    .use(rehypeSlug)
    .use(rehypeHighlight, { detect: false })
    .use(rehypeDocLinks, { file })
    .use(wrapTables)
    .use(collect)
    .use(rehypeStringify)
    .process(markdown);
  return { html: String(html), headings };
}
