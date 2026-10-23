import { deleteLibraryItem, listLibrary, searchLibrary } from "./api";
import type { LibraryItem, SearchHit } from "./types";

/** Everything stored in the library, plus the current full-text search over it. */
class LibraryStore {
  items = $state<LibraryItem[] | null>(null);
  hits = $state<SearchHit[] | null>(null);
  query = $state("");
  #latest = 0;

  async load(): Promise<void> {
    this.items = await listLibrary();
    if (this.query.trim()) await this.search(this.query);
  }

  /** Searches as the user types. Out-of-order answers from earlier keystrokes are dropped. */
  async search(query: string): Promise<void> {
    this.query = query;
    const ticket = ++this.#latest;
    if (!query.trim()) {
      this.hits = null;
      return;
    }
    const hits = await searchLibrary(query);
    if (ticket === this.#latest) this.hits = hits;
  }

  async remove(id: number): Promise<void> {
    await deleteLibraryItem(id);
    await this.load();
  }
}

export const library = new LibraryStore();
