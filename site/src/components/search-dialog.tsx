"use client";

import { useRef, useState } from "react";
import { SearchResults } from "@/components/search-results";
import { SearchTrigger } from "@/components/search-trigger";
import { useSearchIndex } from "@/components/use-search-index";
import { useSearchShortcut } from "@/components/use-search-shortcut";
import { search } from "@/lib/search-rank";

/** A search box over the docs, features, FAQ and models. The index loads the first time it opens. */
export function SearchDialog() {
  const dialog = useRef<HTMLDialogElement>(null);
  const [query, setQuery] = useState("");
  const { index, load } = useSearchIndex();

  const open = () => {
    dialog.current?.showModal();
    load();
  };
  const close = () => dialog.current?.close();
  useSearchShortcut(open);

  return (
    <>
      <SearchTrigger onOpen={open} />
      <dialog
        ref={dialog}
        onClick={(event) => event.target === dialog.current && close()}
        className="m-auto mt-[12vh] w-[min(40rem,92vw)] rounded-md border border-line-strong bg-bg p-0 text-ink backdrop:bg-black/50"
      >
        <input
          autoFocus
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Search the docs, features and FAQ"
          aria-label="Search"
          className="w-full border-b border-line bg-transparent px-4 py-3 outline-none"
        />
        <SearchResults
          hits={index ? search(index, query) : []}
          query={query}
          ready={index !== null}
          onPick={close}
        />
        <p className="border-t border-line px-4 py-2 font-mono text-[11px] tracking-widest text-muted uppercase">
          Esc to close · Ctrl or ⌘ K to open
        </p>
      </dialog>
    </>
  );
}
