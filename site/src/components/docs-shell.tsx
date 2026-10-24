import type { ReactNode } from "react";
import { DocsSidebar } from "@/components/docs-sidebar";
import { PageShell } from "@/components/page-shell";

interface DocsShellProps {
  current?: string;
  toc?: ReactNode;
  children: ReactNode;
}

/** The docs layout: sidebar, content and an optional "on this page" column. */
export function DocsShell({ current, toc, children }: DocsShellProps) {
  return (
    <PageShell>
      <div className="grid lg:grid-cols-[240px_minmax(0,1fr)] xl:grid-cols-[240px_minmax(0,1fr)_220px]">
        <aside className="hidden border-r border-line lg:block">
          <div className="sticky top-14 max-h-[calc(100vh-3.5rem)] overflow-y-auto p-6">
            <DocsSidebar current={current} />
          </div>
        </aside>
        <div className="min-w-0 px-6 py-10 md:px-10">
          <details className="mb-8 rounded-md border border-line lg:hidden">
            <summary className="cursor-pointer px-4 py-2.5 font-mono text-[11px] tracking-widest text-muted uppercase">
              Documentation menu
            </summary>
            <div className="border-t border-line p-4">
              <DocsSidebar current={current} />
            </div>
          </details>
          {children}
        </div>
        {toc ? (
          <aside className="hidden border-l border-line xl:block">
            <div className="sticky top-14 p-6">{toc}</div>
          </aside>
        ) : null}
      </div>
    </PageShell>
  );
}
