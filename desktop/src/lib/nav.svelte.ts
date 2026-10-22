export type PageId = "transcribe" | "speak" | "library" | "models" | "settings";

class Navigation {
  current = $state<PageId>("transcribe");
  /** Library id of the transcript being read, shown instead of the library list. */
  viewing = $state<number | null>(null);

  go(page: PageId): void {
    this.current = page;
    this.viewing = null;
  }

  /** Opens a stored transcript in the viewer. */
  open(itemId: number): void {
    this.current = "library";
    this.viewing = itemId;
  }
}

export const nav = new Navigation();
