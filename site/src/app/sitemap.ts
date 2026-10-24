import type { MetadataRoute } from "next";
import { DOCS } from "@/lib/docs-manifest";
import { FEATURES } from "@/lib/features";
import { SITE_URL } from "@/lib/links";

const PAGES = ["", "/features", "/privacy", "/cli", "/build", "/docs", "/faq"];

export default function sitemap(): MetadataRoute.Sitemap {
  const paths = [
    ...PAGES,
    ...FEATURES.map((f) => `/features/${f.slug}`),
    ...DOCS.map((d) => `/docs/${d.slug}`),
  ];
  return paths.map((path) => ({ url: `${SITE_URL}${path}` }));
}
