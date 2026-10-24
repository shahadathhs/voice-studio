import { DOCS_URL, ISSUES_URL, LICENSE_URL, REPO_URL } from "@/lib/links";

const LINKS = [
  { href: REPO_URL, label: "GitHub" },
  { href: DOCS_URL, label: "Docs" },
  { href: ISSUES_URL, label: "Issues" },
  { href: LICENSE_URL, label: "MIT licence" },
];

export function SiteFooter() {
  return (
    <footer className="border-t border-line">
      <div className="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-3 border-x border-line px-6 py-6 font-mono text-[11px] tracking-widest text-muted uppercase">
        <p>Voice Studio · open source · offline</p>
        <ul className="flex flex-wrap gap-x-5 gap-y-1">
          {LINKS.map((link) => (
            <li key={link.href}>
              <a href={link.href} className="transition-colors duration-150 hover:text-ink">
                {link.label}
              </a>
            </li>
          ))}
        </ul>
      </div>
    </footer>
  );
}
