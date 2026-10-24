export interface DocEntry {
  slug: string;
  /** Path of the markdown file from the repository root; absent for pages built from data. */
  file?: string;
  title: string;
  summary: string;
  group: DocGroup;
}

export type DocGroup = "Use it" | "How it works" | "Contribute";

export const GROUPS: DocGroup[] = ["Use it", "How it works", "Contribute"];

/** The docs section. The markdown files stay the source of truth: the site renders them at build time. */
export const DOCS: DocEntry[] = [
  {
    slug: "building",
    file: "docs/building.md",
    title: "Build from source",
    summary: "Prerequisites, commands, where the app lands, versions.",
    group: "Use it",
  },
  {
    slug: "features",
    file: "docs/product/features.md",
    title: "Features",
    summary: "Every screen and what it does.",
    group: "Use it",
  },
  {
    slug: "cli",
    file: "docs/product/cli.md",
    title: "Command-line reference",
    summary: "Every vtt command, option, exit code and JSON event.",
    group: "Use it",
  },
  {
    slug: "models",
    title: "Models and licences",
    summary: "Every model with its size, languages and licence.",
    group: "Use it",
  },
  {
    slug: "languages",
    title: "Languages",
    summary: "What is supported for transcription, speech, translation and the interface.",
    group: "Use it",
  },
  {
    slug: "localization",
    file: "docs/localization.md",
    title: "Localization",
    summary: "How the interface is translated and how to add a language.",
    group: "Use it",
  },
  {
    slug: "architecture",
    file: "docs/architecture.md",
    title: "Architecture",
    summary: "How the core, CLI and desktop app fit together.",
    group: "How it works",
  },
  {
    slug: "speech",
    file: "docs/product/speech.md",
    title: "Text to speech",
    summary: "How speech is made, the languages and their checks.",
    group: "How it works",
  },
  {
    slug: "speaker-identification",
    file: "docs/product/diarization.md",
    title: "Speaker identification",
    summary: "Method, model, licence, evaluation and limits.",
    group: "How it works",
  },
  {
    slug: "contributing",
    file: "CONTRIBUTING.md",
    title: "Contributing",
    summary: "Setup, workflow and what a good pull request looks like.",
    group: "Contribute",
  },
  {
    slug: "guidelines",
    file: "docs/guidelines.md",
    title: "Code guidelines",
    summary: "Size limits, reuse, errors, tests and commits.",
    group: "Contribute",
  },
  {
    slug: "roadmap",
    file: "docs/product/roadmap.md",
    title: "Roadmap",
    summary: "What is built, what is open, and the risks.",
    group: "Contribute",
  },
];

export function docBySlug(slug: string): DocEntry | undefined {
  return DOCS.find((doc) => doc.slug === slug);
}

export function docByFile(file: string): DocEntry | undefined {
  return DOCS.find((doc) => doc.file === file);
}
