import Link from "next/link";
import { Screenshot } from "@/components/screenshot";
import type { Feature } from "@/lib/features";

export function FeatureCard({ feature, index }: { feature: Feature; index: number }) {
  return (
    <li className="reveal-item">
      <Link
        href={`/features/${feature.slug}`}
        className="group flex h-full flex-col bg-bg transition-colors duration-150 hover:bg-raised"
      >
        <div className="aspect-[16/10] overflow-hidden border-b border-line">
          <div className="h-full transition-transform duration-300 group-hover:scale-[1.03]">
            <Screenshot
              name={feature.shot}
              alt={feature.alt}
              width={feature.width}
              height={feature.height}
              fit
            />
          </div>
        </div>
        <div className="flex flex-1 flex-col p-5">
          <p className="font-mono text-[11px] tracking-widest text-muted uppercase">
            {String(index + 1).padStart(2, "0")} / {feature.name}
          </p>
          <h3 className="mt-2 text-lg font-semibold tracking-tight">{feature.title}</h3>
          <p className="mt-2 flex-1 text-sm text-muted">{feature.summary}</p>
          <p className="mt-4 font-mono text-[11px] tracking-widest text-accent uppercase">
            Read more <span aria-hidden="true">→</span>
          </p>
        </div>
      </Link>
    </li>
  );
}
