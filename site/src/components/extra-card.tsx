import Link from "next/link";

export interface Extra {
  href: string;
  name: string;
  title: string;
  summary: string;
  glyph: string;
}

/** A text-only card that sits beside the feature cards, for pages that are not a screen. */
export function ExtraCard({ extra, index }: { extra: Extra; index: number }) {
  return (
    <li className="reveal-item">
      <Link
        href={extra.href}
        className="group flex h-full flex-col bg-bg transition-colors duration-150 hover:bg-raised"
      >
        <div className="grid aspect-[16/10] place-items-center border-b border-line bg-sunken p-6">
          <pre className="font-mono text-sm leading-relaxed text-accent">{extra.glyph}</pre>
        </div>
        <div className="flex flex-1 flex-col p-5">
          <p className="font-mono text-[11px] tracking-widest text-muted uppercase">
            {String(index).padStart(2, "0")} / {extra.name}
          </p>
          <h3 className="mt-2 text-lg font-semibold tracking-tight">{extra.title}</h3>
          <p className="mt-2 flex-1 text-sm text-muted">{extra.summary}</p>
          <p className="mt-4 font-mono text-[11px] tracking-widest text-accent uppercase">
            Read more <span aria-hidden="true">→</span>
          </p>
        </div>
      </Link>
    </li>
  );
}
