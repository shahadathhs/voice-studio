import type { Metadata } from "next";
import { ButtonLink } from "@/components/button-link";
import { PageShell } from "@/components/page-shell";
import { PageTitle } from "@/components/page-title";

export const metadata: Metadata = { title: "Page not found", robots: { index: false } };

export default function NotFound() {
  return (
    <PageShell>
      <PageTitle eyebrow="404" title="Page not found.">
        <p>That page does not exist.</p>
      </PageTitle>
      <div className="px-6 pb-16 md:px-10">
        <ButtonLink href="/">Back to the home page</ButtonLink>
      </div>
    </PageShell>
  );
}
