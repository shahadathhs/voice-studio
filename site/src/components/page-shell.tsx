import type { ReactNode } from "react";
import { ScrollReveal } from "@/components/scroll-reveal";
import { SiteFooter } from "@/components/site-footer";
import { SiteHeader } from "@/components/site-header";

/** Skip link, scroll progress bar, header, a ruled content column and the footer. */
export function PageShell({ children }: { children: ReactNode }) {
  return (
    <>
      <a
        href="#main"
        className="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-30 focus:rounded-md focus:bg-accent focus:px-3 focus:py-2 focus:text-accent-ink"
      >
        Skip to content
      </a>
      <ScrollReveal />
      <div className="scroll-progress" aria-hidden="true" />
      <SiteHeader />
      <main id="main" className="mx-auto max-w-6xl border-x border-line">
        {children}
      </main>
      <SiteFooter />
    </>
  );
}
