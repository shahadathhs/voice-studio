import Link from "next/link";
import type { SearchHit } from "@/lib/search-rank";

function Snippet({ hit }: { hit: SearchHit }) {
  const { snippet, mark } = hit;
  if (!mark) return <>{snippet}</>;
  return (
    <>
      {snippet.slice(0, mark[0])}
      <mark className="bg-accent-bg text-ink">{snippet.slice(mark[0], mark[1])}</mark>
      {snippet.slice(mark[1])}
    </>
  );
}

interface SearchResultsProps {
  hits: SearchHit[];
  query: string;
  ready: boolean;
  onPick: () => void;
}

export function SearchResults({ hits, query, ready, onPick }: SearchResultsProps) {
  return (
    <ul className="max-h-[60vh] overflow-y-auto">
      {hits.map((hit) => (
        <li
          key={`${hit.entry.url}-${hit.entry.section}`}
          className="border-b border-line last:border-b-0"
        >
          <Link href={hit.entry.url} onClick={onPick} className="block px-4 py-3 hover:bg-raised">
            <p className="font-mono text-[11px] tracking-widest text-muted uppercase">
              {hit.entry.title}
              {hit.entry.section ? ` / ${hit.entry.section}` : ""}
            </p>
            <p className="mt-1 text-sm text-muted">
              <Snippet hit={hit} />
            </p>
          </Link>
        </li>
      ))}
      {query && ready && hits.length === 0 ? (
        <li className="px-4 py-6 text-center text-sm text-muted">
          Nothing found for &ldquo;{query}&rdquo;.
        </li>
      ) : null}
    </ul>
  );
}
