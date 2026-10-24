import Link from "next/link";
import { ThemeToggle } from "@/components/theme-toggle";
import { NAV } from "@/lib/nav";

export function SiteHeader() {
  return (
    <header className="sticky top-0 z-10 border-b border-line bg-bg/90 backdrop-blur">
      <nav className="mx-auto flex h-14 max-w-6xl items-center justify-between gap-4 border-x border-line px-6">
        <Link href="/" className="font-mono text-sm font-semibold tracking-wide">
          VOICE STUDIO
        </Link>
        <div className="flex items-center gap-5">
          <ul className="hidden items-center gap-5 font-mono text-[11px] tracking-widest text-muted uppercase sm:flex">
            {NAV.map((item) => (
              <li key={item.href}>
                <Link href={item.href} className="transition-colors duration-150 hover:text-ink">
                  {item.label}
                </Link>
              </li>
            ))}
          </ul>
          <ThemeToggle />
        </div>
      </nav>
    </header>
  );
}
