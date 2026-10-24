import Link from "next/link";
import { Breadcrumbs } from "@/components/breadcrumbs";
import { JsonLd } from "@/components/json-ld";
import { PageShell } from "@/components/page-shell";
import { PageTitle } from "@/components/page-title";
import { FAQS } from "@/lib/faq";
import { pageMetadata } from "@/lib/seo";
import { faqSchema } from "@/lib/structured-data";

export const metadata = pageMetadata({
  title: "FAQ",
  description:
    "Answers about Voice Studio: is it offline, why there are no installers, which model to start with, GPU support, languages, voice cloning and licences.",
  path: "/faq",
});

export default function FaqPage() {
  return (
    <PageShell>
      <JsonLd data={faqSchema(FAQS)} />
      <div className="px-6 pt-8 md:px-10">
        <Breadcrumbs trail={[{ name: "Home", href: "/" }, { name: "FAQ" }]} />
      </div>
      <PageTitle eyebrow="FAQ" title="Questions people ask.">
        <p>Short answers, with a link to the full page where there is one.</p>
      </PageTitle>
      <ul className="reveal-group border-t border-line">
        {FAQS.map((faq) => (
          <li key={faq.question} className="reveal-item border-b border-line">
            <details className="group px-6 py-5 md:px-10">
              <summary className="flex cursor-pointer list-none items-center justify-between gap-4 font-medium [&::-webkit-details-marker]:hidden">
                {faq.question}
                <span
                  className="font-mono text-accent transition-transform duration-150 group-open:rotate-45"
                  aria-hidden="true"
                >
                  +
                </span>
              </summary>
              <p className="mt-3 max-w-2xl text-muted">{faq.answer}</p>
              <p className="mt-3">
                <Link
                  href={faq.href}
                  className="font-mono text-[11px] tracking-widest text-accent uppercase underline-offset-4 hover:underline"
                >
                  {faq.more} →
                </Link>
              </p>
            </details>
          </li>
        ))}
      </ul>
    </PageShell>
  );
}
