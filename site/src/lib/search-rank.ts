import type { SearchEntry } from "@/lib/search-index";

export interface SearchHit {
  entry: SearchEntry;
  snippet: string;
  /** Start and end of the first match inside the snippet, for highlighting. */
  mark: [number, number] | null;
}

const MAX_RESULTS = 8;

function score(entry: SearchEntry, terms: string[]): number {
  const title = entry.title.toLowerCase();
  const section = entry.section.toLowerCase();
  const text = entry.text.toLowerCase();
  let total = 0;
  for (const term of terms) {
    const inTitle = title.includes(term);
    const inSection = section.includes(term);
    const inText = text.split(term).length - 1;
    if (!inTitle && !inSection && inText === 0) return 0;
    total += (inTitle ? 10 : 0) + (inSection ? 6 : 0) + Math.min(inText, 5);
  }
  return total;
}

function snippetOf(entry: SearchEntry, term: string): Pick<SearchHit, "snippet" | "mark"> {
  const at = entry.text.toLowerCase().indexOf(term);
  if (at < 0) return { snippet: entry.text.slice(0, 140), mark: null };
  const start = Math.max(0, at - 50);
  const snippet = (start > 0 ? "…" : "") + entry.text.slice(start, at + 110);
  const from = snippet.toLowerCase().indexOf(term);
  return { snippet, mark: [from, from + term.length] };
}

/** Entries that contain every word of the query, best first. */
export function search(index: SearchEntry[], query: string): SearchHit[] {
  const terms = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (terms.length === 0) return [];
  return index
    .map((entry) => ({ entry, points: score(entry, terms) }))
    .filter((hit) => hit.points > 0)
    .sort((a, b) => b.points - a.points)
    .slice(0, MAX_RESULTS)
    .map(({ entry }) => ({ entry, ...snippetOf(entry, terms[0]) }));
}
