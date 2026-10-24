import type { ReactNode } from "react";

interface ButtonLinkProps {
  href: string;
  children: ReactNode;
  quiet?: boolean;
}

/** One filled primary button per screenful; secondary actions are quiet text links. */
export function ButtonLink({ href, children, quiet }: ButtonLinkProps) {
  const style = quiet
    ? "text-muted underline underline-offset-4 hover:text-ink"
    : "bg-accent text-accent-ink px-5 py-2.5 font-medium hover:opacity-90";
  return (
    <a href={href} className={`inline-block rounded-md text-sm ${style}`}>
      {children}
    </a>
  );
}
