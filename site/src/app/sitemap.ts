import type { MetadataRoute } from "next";
import { SITE_URL } from "@/lib/links";
import { NAV } from "@/lib/nav";

export default function sitemap(): MetadataRoute.Sitemap {
  return ["", ...NAV.map((item) => item.href)].map((path) => ({ url: `${SITE_URL}${path}` }));
}
