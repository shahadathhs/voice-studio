import type { ReactNode } from "react";
import { SectionFrame } from "@/components/section-frame";

interface BandProps {
  index: string;
  name: string;
  tag?: string;
  title: ReactNode;
  children: ReactNode;
  action?: ReactNode;
}

/** A home-page section: label bar, headline, copy and one action. */
export function Band({ index, name, tag, title, children, action }: BandProps) {
  return (
    <SectionFrame index={index} name={name} tag={tag}>
      <div className="reveal px-6 py-14 md:px-10">
        <h2 className="max-w-2xl text-2xl font-semibold tracking-tight sm:text-4xl">{title}</h2>
        <div className="mt-4 max-w-2xl text-muted">{children}</div>
        {action ? <div className="mt-7">{action}</div> : null}
      </div>
    </SectionFrame>
  );
}
