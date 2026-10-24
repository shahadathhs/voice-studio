import type { Metadata } from "next";
import { SITE_URL } from "@/lib/links";

export const SITE_NAME = "Voice Studio";

export const SITE_DESCRIPTION =
  "Turn speech into text, and text into speech, on your own computer. A free, open source desktop app and command-line tool for Windows, macOS and Linux. No account, no cloud.";

interface PageSeo {
  title: string;
  description: string;
  path: string;
}

/** Title, description, canonical URL and social cards for one page. */
export function pageMetadata({ title, description, path }: PageSeo): Metadata {
  const url = `${SITE_URL}${path}`;
  return {
    title,
    description,
    alternates: { canonical: path || "/" },
    openGraph: {
      title: `${title} · ${SITE_NAME}`,
      description,
      url,
      type: "website",
      siteName: SITE_NAME,
    },
    twitter: { card: "summary_large_image", title: `${title} · ${SITE_NAME}`, description },
  };
}
