import type { MetadataRoute } from "next";
import { SITE_URL } from "@/lib/links";

const PATHS = ["", "/build"];

export default function sitemap(): MetadataRoute.Sitemap {
  return PATHS.map((path) => ({ url: `${SITE_URL}${path}` }));
}
