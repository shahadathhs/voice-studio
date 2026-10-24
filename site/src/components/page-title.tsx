import type { ReactNode } from "react";

interface PageTitleProps {
  eyebrow: string;
  title: ReactNode;
  children?: ReactNode;
}

/** The opening block of an inner page: mono eyebrow, headline, lede. */
export function PageTitle({ eyebrow, title, children }: PageTitleProps) {
  return (
    <header className="reveal px-6 pt-12 pb-10 md:px-10">
      <p className="font-mono text-[11px] tracking-widest text-muted uppercase">{eyebrow}</p>
      <h1 className="mt-3 max-w-3xl text-3xl font-semibold tracking-tight sm:text-5xl">{title}</h1>
      {children ? <div className="mt-5 max-w-2xl text-lg text-muted">{children}</div> : null}
    </header>
  );
}
