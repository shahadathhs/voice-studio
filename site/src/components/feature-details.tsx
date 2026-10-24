import { BulletList } from "@/components/bullet-list";
import { CodeBlock } from "@/components/code-block";
import { SectionFrame } from "@/components/section-frame";
import type { Feature } from "@/lib/features";

export function FeatureDetails({ feature }: { feature: Feature }) {
  return (
    <>
      <SectionFrame index="01" name="What it does">
        <div className="reveal grid gap-8 px-6 py-10 md:grid-cols-2 md:px-10">
          <BulletList items={feature.points} />
          <div>
            <h2 className="font-mono text-[11px] tracking-widest text-muted uppercase">
              How it works
            </h2>
            <ol className="mt-3 list-decimal space-y-2 pl-5 text-sm text-muted">
              {feature.how.map((step) => (
                <li key={step}>{step}</li>
              ))}
            </ol>
          </div>
        </div>
      </SectionFrame>
      <SectionFrame index="02" name="In the command line" tag="vtt">
        <div className="reveal px-6 py-10 md:px-10">
          <CodeBlock code={feature.cli} />
        </div>
      </SectionFrame>
      <SectionFrame index="03" name="Honest limits">
        <div className="reveal px-6 py-10 md:px-10">
          <BulletList items={feature.limits} />
        </div>
      </SectionFrame>
    </>
  );
}
