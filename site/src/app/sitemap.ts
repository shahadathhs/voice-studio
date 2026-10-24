import type { MetadataRoute } from "next";
import { FEATURES } from "@/lib/features";
import { SITE_URL } from "@/lib/links";

const PAGES = ["", "/features", "/privacy", "/cli", "/build"];

export default function sitemap(): MetadataRoute.Sitemap {
  const paths = [...PAGES, ...FEATURES.map((f) => `/features/${f.slug}`)];
  return paths.map((path) => ({ url: `${SITE_URL}${path}` }));
}
