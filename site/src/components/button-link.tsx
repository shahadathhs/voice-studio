import type { ReactNode } from "react";

interface ButtonLinkProps {
  href: string;
  children: ReactNode;
  quiet?: boolean;
}

/** One filled primary button per screenful; other actions are quiet mono links. */
export function ButtonLink({ href, children, quiet }: ButtonLinkProps) {
  const style = quiet
    ? "border border-line-strong text-ink hover:bg-raised"
    : "bg-accent text-accent-ink hover:opacity-90";
  return (
    <a
      href={href}
      className={`inline-block rounded-md px-5 py-2.5 font-mono text-xs font-medium tracking-widest uppercase transition-colors duration-150 ${style}`}
    >
      {children}
    </a>
  );
}
