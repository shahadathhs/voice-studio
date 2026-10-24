import Link from "next/link";
import { SectionFrame } from "@/components/section-frame";
import { featureBySlug } from "@/lib/features";

export function RelatedFeatures({ slugs }: { slugs: string[] }) {
  const items = slugs.flatMap((slug) => featureBySlug(slug) ?? []);
  return (
    <SectionFrame index="04" name="Related">
      <ul className="reveal-group grid gap-px bg-line sm:grid-cols-2">
        {items.map((feature) => (
          <li key={feature.slug} className="reveal-item bg-bg">
            <Link
              href={`/features/${feature.slug}`}
              className="block p-6 transition-colors duration-150 hover:bg-raised md:px-10"
            >
              <p className="font-mono text-[11px] tracking-widest text-muted uppercase">
                {feature.name}
              </p>
              <p className="mt-2 font-medium">{feature.title}</p>
              <p className="mt-1 text-sm text-muted">{feature.summary}</p>
            </Link>
          </li>
        ))}
      </ul>
    </SectionFrame>
  );
}
