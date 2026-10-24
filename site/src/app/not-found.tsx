import { ButtonLink } from "@/components/button-link";
import { PageShell } from "@/components/page-shell";

export default function NotFound() {
  return (
    <PageShell>
      <h1 className="text-3xl font-semibold tracking-tight">Page not found</h1>
      <div className="mt-6">
        <ButtonLink href="/">Back to the home page</ButtonLink>
      </div>
    </PageShell>
  );
}
