import GithubSlugger from "github-slugger";
import { DOCS } from "@/lib/docs-manifest";
import { readRepoFile, withoutTitle } from "@/lib/docs-source";
import { FAQS } from "@/lib/faq";
import { FEATURES } from "@/lib/features";
import { readCatalog } from "@/lib/models-data";

export interface SearchEntry {
  /** Page title. */
  title: string;
  /** Section heading inside the page, if any. */
  section: string;
  url: string;
  text: string;
}

/** Markdown reduced to the words a person would search for. Code is kept: commands are searched too. */
function plain(markdown: string): string {
  return markdown
    .replace(/```[a-z]*\n?/g, " ")
    .replace(/!?\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/[`*_>|#]/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

/** One entry per `##` section of a markdown file, linked to the section's own anchor. */
function markdownEntries(title: string, url: string, markdown: string): SearchEntry[] {
  const slugger = new GithubSlugger();
  const entries: SearchEntry[] = [];
  let section = "";
  let id = "";
  let lines: string[] = [];
  let inFence = false;
  const flush = () => {
    const text = plain(lines.join("\n"));
    if (text) entries.push({ title, section, url: id ? `${url}#${id}` : url, text });
    lines = [];
  };
  for (const line of markdown.split("\n")) {
    if (line.startsWith("```")) inFence = !inFence;
    const heading = !inFence && /^#{2,3} (.*)/.exec(line);
    if (heading && line.startsWith("## ")) {
      flush();
      section = plain(heading[1]);
      id = slugger.slug(heading[1].replace(/[`*_]/g, ""));
    } else {
      if (heading) slugger.slug(heading[1].replace(/[`*_]/g, ""));
      lines.push(line);
    }
  }
  flush();
  return entries;
}

function docEntries(): SearchEntry[] {
  return DOCS.flatMap((doc) =>
    doc.file
      ? markdownEntries(doc.title, `/docs/${doc.slug}`, withoutTitle(readRepoFile(doc.file)))
      : [],
  );
}

function featureEntries(): SearchEntry[] {
  return FEATURES.map((f) => ({
    title: f.name,
    section: f.title,
    url: `/features/${f.slug}`,
    text: [f.intro, ...f.points, ...f.how, ...f.limits, f.cli].join(" "),
  }));
}

function faqEntries(): SearchEntry[] {
  return FAQS.map((f) => ({ title: "FAQ", section: f.question, url: "/faq", text: f.answer }));
}

function modelEntries(): SearchEntry[] {
  return readCatalog().map((m) => ({
    title: "Models and licences",
    section: m.name,
    url: "/docs/models",
    text: `${m.description} ${m.id} ${m.license} ${m.languages.join(" ")}`,
  }));
}

/** Everything the search box can find, built at deploy time. */
export function buildSearchIndex(): SearchEntry[] {
  return [...featureEntries(), ...faqEntries(), ...docEntries(), ...modelEntries()];
}
