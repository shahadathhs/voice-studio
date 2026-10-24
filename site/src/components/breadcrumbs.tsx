import Link from "next/link";

interface Crumb {
  name: string;
  href?: string;
}

export function Breadcrumbs({ trail }: { trail: Crumb[] }) {
  return (
    <nav
      aria-label="Breadcrumb"
      className="font-mono text-[11px] tracking-widest text-muted uppercase"
    >
      <ol className="flex flex-wrap gap-2">
        {trail.map((crumb, i) => (
          <li key={crumb.name} className="flex gap-2">
            {crumb.href ? (
              <Link href={crumb.href} className="hover:text-ink">
                {crumb.name}
              </Link>
            ) : (
              <span aria-current="page" className="text-ink">
                {crumb.name}
              </span>
            )}
            {i < trail.length - 1 ? <span aria-hidden="true">/</span> : null}
          </li>
        ))}
      </ol>
    </nav>
  );
}
