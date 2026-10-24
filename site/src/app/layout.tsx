import type { Metadata, Viewport } from "next";
import type { ReactNode } from "react";
import { ThemeScript } from "@/components/theme-script";
import { SITE_URL } from "@/lib/links";
import { SITE_DESCRIPTION, SITE_NAME } from "@/lib/seo";
import "./globals.css";

export const metadata: Metadata = {
  metadataBase: new URL(SITE_URL),
  title: {
    default: `${SITE_NAME}: offline speech to text and text to speech`,
    template: `%s · ${SITE_NAME}`,
  },
  description: SITE_DESCRIPTION,
  applicationName: SITE_NAME,
  keywords: [
    "speech to text",
    "text to speech",
    "offline transcription",
    "Whisper",
    "speaker diarization",
    "voice cloning",
    "open source",
  ],
  openGraph: { siteName: SITE_NAME, type: "website", locale: "en_US" },
  twitter: { card: "summary_large_image" },
  robots: { index: true, follow: true },
};

export const viewport: Viewport = {
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#f4f3f0" },
    { media: "(prefers-color-scheme: dark)", color: "#171716" },
  ],
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <ThemeScript />
      </head>
      <body>{children}</body>
    </html>
  );
}
