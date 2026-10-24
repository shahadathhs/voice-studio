import Link from "next/link";
import { LogoMark } from "@/components/logo-mark";
import { FEATURES } from "@/lib/features";
import { DOCS_URL, ISSUES_URL, LICENSE_URL, REPO_URL } from "@/lib/links";

const COLUMNS = [
  {
    title: "Features",
    links: FEATURES.map((f) => ({ href: `/features/${f.slug}`, label: f.name })),
  },
  {
    title: "Product",
    links: [
      { href: "/privacy", label: "Privacy" },
      { href: "/cli", label: "Command line" },
      { href: "/build", label: "Build from source" },
    ],
  },
  {
    title: "Project",
    links: [
      { href: REPO_URL, label: "GitHub" },
      { href: DOCS_URL, label: "Documentation" },
      { href: ISSUES_URL, label: "Issues" },
      { href: LICENSE_URL, label: "MIT licence" },
    ],
  },
];

export function SiteFooter() {
  return (
    <footer className="border-t border-line">
      <div className="mx-auto grid max-w-6xl gap-10 border-x border-line px-6 py-10 sm:grid-cols-2 lg:grid-cols-4">
        <div>
          <p className="flex items-center gap-2.5 font-mono text-sm font-semibold">
            <LogoMark />
            VOICE STUDIO
          </p>
          <p className="mt-3 max-w-xs text-sm text-muted">
            Speech to text and back, on your own computer. Open source, offline.
          </p>
        </div>
        {COLUMNS.map((column) => (
          <nav key={column.title} aria-label={column.title}>
            <h2 className="font-mono text-[11px] tracking-widest text-muted uppercase">
              {column.title}
            </h2>
            <ul className="mt-3 space-y-2 text-sm">
              {column.links.map((link) => (
                <li key={link.href}>
                  <Link
                    href={link.href}
                    className="text-muted transition-colors duration-150 hover:text-ink"
                  >
                    {link.label}
                  </Link>
                </li>
              ))}
            </ul>
          </nav>
        ))}
      </div>
    </footer>
  );
}
