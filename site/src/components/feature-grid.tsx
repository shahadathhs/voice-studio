import { FEATURES } from "@/lib/features";

export function FeatureGrid() {
  return (
    <section className="mt-20">
      <h2 className="text-2xl font-semibold tracking-tight">What you can do</h2>
      <ul className="mt-8 grid gap-8 sm:grid-cols-2 lg:grid-cols-3">
        {FEATURES.map((feature) => (
          <li key={feature.title}>
            <h3 className="font-medium">{feature.title}</h3>
            <p className="mt-2 text-sm text-muted">{feature.body}</p>
          </li>
        ))}
      </ul>
    </section>
  );
}
