import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { Breadcrumbs } from "@/components/breadcrumbs";
import { ButtonLink } from "@/components/button-link";
import { FeatureDetails } from "@/components/feature-details";
import { JsonLd } from "@/components/json-ld";
import { PageShell } from "@/components/page-shell";
import { PageTitle } from "@/components/page-title";
import { RelatedFeatures } from "@/components/related-features";
import { Screenshot } from "@/components/screenshot";
import { FEATURES, featureBySlug } from "@/lib/features";
import { pageMetadata } from "@/lib/seo";
import { breadcrumbs } from "@/lib/structured-data";

type Props = { params: Promise<{ slug: string }> };

export function generateStaticParams() {
  return FEATURES.map((feature) => ({ slug: feature.slug }));
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const feature = featureBySlug((await params).slug);
  if (!feature) return {};
  return pageMetadata({
    title: feature.name,
    description: feature.description,
    path: `/features/${feature.slug}`,
  });
}

export default async function FeaturePage({ params }: Props) {
  const feature = featureBySlug((await params).slug);
  if (!feature) notFound();
  const path = `/features/${feature.slug}`;
  return (
    <PageShell>
      <JsonLd
        data={breadcrumbs([
          { name: "Voice Studio", path: "/" },
          { name: "Features", path: "/features" },
          { name: feature.name, path },
        ])}
      />
      <div className="px-6 pt-8 md:px-10">
        <Breadcrumbs
          trail={[
            { name: "Home", href: "/" },
            { name: "Features", href: "/features" },
            { name: feature.name },
          ]}
        />
      </div>
      <PageTitle eyebrow={feature.name} title={feature.title}>
        <p>{feature.intro}</p>
      </PageTitle>
      <div className="border-t border-line bg-sunken p-4 sm:p-8">
        <div className="hero-shot mx-auto max-w-5xl overflow-hidden rounded-md border border-line-strong">
          <Screenshot
            name={feature.shot}
            alt={feature.alt}
            width={feature.width}
            height={feature.height}
            priority
          />
        </div>
      </div>
      <FeatureDetails feature={feature} />
      <RelatedFeatures slugs={feature.related} />
      <div className="reveal border-t border-line px-6 py-10 text-center md:px-10">
        <ButtonLink href="/build">Build from source</ButtonLink>
      </div>
    </PageShell>
  );
}
