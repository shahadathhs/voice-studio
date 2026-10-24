import type { Metadata } from "next";
import type { ReactNode } from "react";
import { SITE_URL } from "@/lib/links";
import "./globals.css";

const DESCRIPTION =
  "Turn speech into text, and text into speech, on your own computer. A free, open source desktop app and command-line tool. No account, no cloud.";

export const metadata: Metadata = {
  metadataBase: new URL(SITE_URL),
  title: {
    default: "Voice Studio: offline speech to text and text to speech",
    template: "%s · Voice Studio",
  },
  description: DESCRIPTION,
  openGraph: {
    title: "Voice Studio",
    description: DESCRIPTION,
    type: "website",
    siteName: "Voice Studio",
  },
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
