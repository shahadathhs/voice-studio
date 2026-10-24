export function SearchTrigger({ onOpen }: { onOpen: () => void }) {
  return (
    <button
      type="button"
      onClick={onOpen}
      aria-label="Search the site"
      className="flex h-8 items-center gap-2 rounded-md border border-line px-2.5 font-mono text-[11px] tracking-widest text-muted uppercase transition-colors duration-150 hover:bg-raised hover:text-ink"
    >
      <svg
        className="size-4"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        aria-hidden="true"
      >
        <circle cx="11" cy="11" r="7" />
        <path d="m20 20-3.5-3.5" />
      </svg>
      <span className="hidden lg:inline">Search</span>
    </button>
  );
}
