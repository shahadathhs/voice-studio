import { LOGO_BARS } from "@/lib/logo-bars";

const WIDTH = 2.6;
const PITCH = 3.6;

/** The Voice Studio mark: a rounded square holding a sound wave that dips toward the middle. */
export function LogoMark({ className = "size-7" }: { className?: string }) {
  const total = PITCH * (LOGO_BARS.length - 1) + WIDTH;
  const start = (32 - total) / 2;
  return (
    <svg className={className} viewBox="0 0 32 32" aria-hidden="true">
      <rect width="32" height="32" rx="7" className="fill-accent" />
      {LOGO_BARS.map((height, i) => (
        <rect
          key={i}
          x={start + i * PITCH}
          y={(32 - height) / 2}
          width={WIDTH}
          height={height}
          rx={WIDTH / 2}
          className="fill-accent-ink"
        />
      ))}
    </svg>
  );
}
