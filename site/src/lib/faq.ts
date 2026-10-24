export interface Faq {
  question: string;
  answer: string;
  /** Where to read more. */
  href: string;
  more: string;
}

/** Keep in step with README.md and the docs. Every answer is something the repository states. */
export const FAQS: Faq[] = [
  {
    question: "Is it really offline?",
    answer:
      "Yes. Transcription, speaker labels, speech, voice cloning, summaries and translation all run on your computer. The only network use is downloading models, which you start and which are checked against a published checksum. There is no account and no telemetry.",
    href: "/privacy",
    more: "Privacy",
  },
  {
    question: "Why are there no installers to download?",
    answer:
      "Prebuilt installers need code signing, and unsigned ones are blocked or warned about by macOS and Windows. Signing needs paid certificates. A copy you build yourself starts normally, so Voice Studio is built from source for now.",
    href: "/build",
    more: "Build from source",
  },
  {
    question: "Which speech model should I start with?",
    answer:
      "Whisper small is the best balance for most people: it copes with accents and some background noise at 488 MB. Tiny and base are for quick drafts of clear speech. Medium and the large models are for difficult audio and are slower.",
    href: "/docs/models",
    more: "Models and licences",
  },
  {
    question: "Does it use my GPU?",
    answer:
      "On macOS, Whisper uses the GPU (Metal) automatically. On Windows and Linux it uses the CPU unless you build with the optional Vulkan or CUDA feature, which needs the matching SDK.",
    href: "/docs/building",
    more: "Build from source",
  },
  {
    question: "Which systems does it run on?",
    answer:
      "Windows, macOS and Linux. The checks run on all three for every change. The app has not yet been confirmed by hand on real Windows and Linux machines, and the roadmap says so.",
    href: "/docs/roadmap",
    more: "Roadmap",
  },
  {
    question: "Which languages are supported?",
    answer:
      "Transcription uses multilingual Whisper models. Speech covers seven languages with 35 voices, cloning covers six, summaries and translation cover 20, and the interface is in English and Spanish. No native speaker has reviewed the speech or translations yet.",
    href: "/docs/languages",
    more: "Languages",
  },
  {
    question: "Can I clone any voice?",
    answer:
      "Only a voice you own or have permission to use. The app asks you to confirm that, marks cloned voices, lets you delete them, and adds a synthetic voice note to saved audio. There is no watermark, and no third-party or celebrity voices are bundled.",
    href: "/features/voices",
    more: "Voices",
  },
  {
    question: "How accurate are the summaries and translations?",
    answer:
      "A small model on your computer writes them, so they can contain errors or invented details. They are labelled as machine-written and never replace the transcript. Bengali output is unreliable and is flagged in the app.",
    href: "/features/library",
    more: "Library",
  },
  {
    question: "How much disk space and memory does it need?",
    answer:
      "Models are downloaded as you need them: from 78 MB (Whisper tiny) to 3.1 GB (Whisper large-v3), about 1.6 GB for voice cloning, and 1.1 or 4.7 GB for the language models. The larger language model needs about 6 GB of memory.",
    href: "/docs/models",
    more: "Models and licences",
  },
  {
    question: "Does dictation type into other apps?",
    answer:
      "No. A global shortcut starts and stops dictation and the text is copied to the clipboard, ready to paste. Typing into other apps would need accessibility permissions on every system.",
    href: "/features/record",
    more: "Record",
  },
  {
    question: "Can I use it from scripts?",
    answer:
      "Yes. The vtt command-line tool does what the app does, with machine-readable JSON output, documented exit codes and shell completions.",
    href: "/cli",
    more: "Command line",
  },
  {
    question: "Is it free? What is the licence?",
    answer:
      "It is free and open source under the MIT licence, with no account and no per-minute fees. Each model has its own licence, and only permissive ones are listed.",
    href: "/docs/models",
    more: "Models and licences",
  },
  {
    question: "How do I report a bug or help?",
    answer:
      "Open an issue on GitHub, or read the contributing guide for setup and what a good pull request looks like.",
    href: "/docs/contributing",
    more: "Contributing",
  },
];
