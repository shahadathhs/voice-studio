import { DOCS_URL, ISSUES_URL, LICENSE_URL, REPO_URL } from "@/lib/links";

const LINKS = [
  { href: REPO_URL, label: "GitHub" },
  { href: DOCS_URL, label: "Documentation" },
  { href: ISSUES_URL, label: "Issues" },
  { href: LICENSE_URL, label: "MIT licence" },
];

export function SiteFooter() {
  return (
    <footer className="border-t border-line text-sm text-muted">
      <div className="mx-auto flex max-w-5xl flex-wrap items-center justify-between gap-3 px-4 py-6">
        <p>Voice Studio. Open source, offline, yours.</p>
        <ul className="flex flex-wrap gap-x-5 gap-y-1">
          {LINKS.map((link) => (
            <li key={link.href}>
              <a href={link.href} className="hover:text-ink">
                {link.label}
              </a>
            </li>
          ))}
        </ul>
      </div>
    </footer>
  );
}
