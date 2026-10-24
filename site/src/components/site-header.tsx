import Link from "next/link";
import { LogoMark } from "@/components/logo-mark";
import { MobileMenu } from "@/components/mobile-menu";
import { ThemeToggle } from "@/components/theme-toggle";
import { NAV } from "@/lib/nav";

const LINK = "transition-colors duration-150 hover:text-ink";

export function SiteHeader() {
  return (
    <header className="sticky top-0 z-10 border-b border-line bg-bg/90 backdrop-blur">
      <nav
        aria-label="Main"
        className="relative mx-auto flex h-14 max-w-6xl items-center justify-between gap-4 border-x border-line px-6"
      >
        <Link
          href="/"
          className="flex items-center gap-2.5 font-mono text-sm font-semibold tracking-wide"
        >
          <LogoMark />
          VOICE STUDIO
        </Link>
        <div className="flex items-center gap-4">
          <ul className="hidden items-center gap-5 font-mono text-[11px] tracking-widest text-muted uppercase md:flex">
            {NAV.map((item) => (
              <li key={item.href}>
                <Link href={item.href} className={LINK}>
                  {item.label}
                </Link>
              </li>
            ))}
          </ul>
          <ThemeToggle />
          <MobileMenu />
        </div>
      </nav>
    </header>
  );
}
