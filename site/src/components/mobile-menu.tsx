import Link from "next/link";
import { NAV } from "@/lib/nav";

/** A menu for small screens that works without JavaScript (a details element). */
export function MobileMenu() {
  return (
    <details className="md:hidden">
      <summary className="grid size-8 cursor-pointer list-none place-items-center rounded-md border border-line text-muted [&::-webkit-details-marker]:hidden">
        <span className="sr-only">Menu</span>
        <svg
          className="size-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          aria-hidden="true"
        >
          <path d="M4 7h16M4 12h16M4 17h16" />
        </svg>
      </summary>
      <ul className="absolute top-14 right-0 left-0 border-y border-line bg-bg px-6 py-3 font-mono text-xs tracking-widest uppercase">
        {NAV.map((item) => (
          <li key={item.href}>
            <Link href={item.href} className="block py-2.5 text-muted hover:text-ink">
              {item.label}
            </Link>
          </li>
        ))}
      </ul>
    </details>
  );
}
