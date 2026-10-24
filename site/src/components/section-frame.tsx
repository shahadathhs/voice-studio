import type { ReactNode } from "react";

interface SectionFrameProps {
  id?: string;
  index: string;
  name: string;
  tag?: string;
  children: ReactNode;
}

/** A ruled section with a small numbered label bar, like a drawing sheet. */
export function SectionFrame({ id, index, name, tag, children }: SectionFrameProps) {
  return (
    <section id={id} className="scroll-mt-14 border-t border-line">
      <div className="section-bar relative flex justify-between gap-4 border-b border-line px-6 py-2 font-mono text-[11px] tracking-widest text-muted uppercase">
        <span>
          {index} / {name}
        </span>
        {tag ? <span>{tag}</span> : null}
      </div>
      {children}
    </section>
  );
}
