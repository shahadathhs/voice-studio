import { ExtraCard, type Extra } from "@/components/extra-card";
import { FeatureCard } from "@/components/feature-card";
import { FEATURES } from "@/lib/features";

/** Every feature as a card that links to its own page, then any extra cards. */
export function FeatureGrid({ extras = [] }: { extras?: Extra[] }) {
  return (
    <ul className="reveal-group grid gap-px bg-line sm:grid-cols-2 lg:grid-cols-3">
      {FEATURES.map((feature, i) => (
        <FeatureCard key={feature.slug} feature={feature} index={i} />
      ))}
      {extras.map((extra, i) => (
        <ExtraCard key={extra.href} extra={extra} index={FEATURES.length + i + 1} />
      ))}
    </ul>
  );
}
