import { ButtonLink } from "@/components/button-link";
import { PageShell } from "@/components/page-shell";

export default function NotFound() {
  return (
    <PageShell>
      <div className="min-h-[60vh] px-6 py-12 md:px-10">
        <h1 className="text-3xl font-semibold tracking-tight">Page not found</h1>
        <div className="mt-6">
          <ButtonLink href="/">Back to the home page</ButtonLink>
        </div>
      </div>
    </PageShell>
  );
}
