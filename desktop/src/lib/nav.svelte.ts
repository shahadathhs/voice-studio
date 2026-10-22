export type PageId = "transcribe" | "speak" | "library" | "models" | "settings";

class Navigation {
  current = $state<PageId>("transcribe");

  go(page: PageId): void {
    this.current = page;
  }
}

export const nav = new Navigation();
