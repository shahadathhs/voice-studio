import type { Extra } from "@/components/extra-card";

export const EXTRAS: Extra[] = [
  {
    href: "/cli",
    name: "Command line",
    title: "The same engine, for scripts.",
    summary: "vtt does what the app does, with JSON output and documented exit codes.",
    glyph: '$ vtt transcribe talk.m4a\n$ vtt speak "Hello" -o hello.wav',
  },
  {
    href: "/privacy",
    name: "Private by design",
    title: "Your audio stays yours.",
    summary: "No account, no telemetry. The only network use is downloading models you choose.",
    glyph: "uploaded: 0 bytes\ntelemetry: none",
  },
];
