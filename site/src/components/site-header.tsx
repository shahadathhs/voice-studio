import Link from "next/link";
import { NAV } from "@/lib/nav";

export function SiteHeader() {
  return (
    <header className="border-b border-line">
      <nav className="mx-auto flex max-w-5xl items-center justify-between gap-4 px-4 py-4">
        <Link href="/" className="font-semibold tracking-tight">
          Voice Studio
        </Link>
        <ul className="flex flex-wrap items-center gap-x-5 gap-y-1 text-sm text-muted">
          {NAV.map((item) => (
            <li key={item.href}>
              <Link href={item.href} className="hover:text-ink">
                {item.label}
              </Link>
            </li>
          ))}
        </ul>
      </nav>
    </header>
  );
}
