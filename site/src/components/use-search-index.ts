"use client";

import { useState } from "react";
import type { SearchEntry } from "@/lib/search-index";

/** The search index, fetched the first time `load` is called. */
export function useSearchIndex() {
  const [index, setIndex] = useState<SearchEntry[] | null>(null);
  const load = () => {
    if (index !== null) return;
    fetch("/search-index.json")
      .then((res) => res.json() as Promise<SearchEntry[]>)
      .then(setIndex)
      .catch(() => setIndex([]));
  };
  return { index, load };
}
