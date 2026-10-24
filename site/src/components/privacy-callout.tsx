import { ButtonLink } from "@/components/button-link";

export function PrivacyCallout() {
  return (
    <section className="mt-20 rounded-md border border-line bg-surface p-6">
      <h2 className="text-2xl font-semibold tracking-tight">Private by design</h2>
      <p className="mt-3 max-w-2xl text-muted">
        Audio and text never leave your computer. The only network use is downloading models, which
        you start yourself, and each download is checked against a published checksum. The app has
        no telemetry.
      </p>
      <div className="mt-5">
        <ButtonLink href="/privacy" quiet>
          How privacy works
        </ButtonLink>
      </div>
    </section>
  );
}
