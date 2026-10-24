import { REPO_URL, SITE_URL } from "@/lib/links";
import { SITE_DESCRIPTION, SITE_NAME } from "@/lib/seo";

export const APP_SCHEMA = {
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  name: SITE_NAME,
  description: SITE_DESCRIPTION,
  url: SITE_URL,
  applicationCategory: "MultimediaApplication",
  operatingSystem: "Windows, macOS, Linux",
  offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
  license: "https://opensource.org/license/mit",
  codeRepository: REPO_URL,
  isAccessibleForFree: true,
};

export function breadcrumbs(trail: { name: string; path: string }[]) {
  return {
    "@context": "https://schema.org",
    "@type": "BreadcrumbList",
    itemListElement: trail.map((item, i) => ({
      "@type": "ListItem",
      position: i + 1,
      name: item.name,
      item: `${SITE_URL}${item.path}`,
    })),
  };
}
