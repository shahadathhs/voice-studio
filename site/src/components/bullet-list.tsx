export function BulletList({ items }: { items: string[] }) {
  return (
    <ul className="space-y-2 text-sm text-muted">
      {items.map((item) => (
        <li key={item} className="flex gap-3">
          <span className="mt-2 size-1.5 shrink-0 bg-accent" aria-hidden="true" />
          {item}
        </li>
      ))}
    </ul>
  );
}
