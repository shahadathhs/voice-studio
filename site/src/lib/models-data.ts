import { readRepoFile } from "@/lib/docs-source";

export interface CatalogModel {
  id: string;
  name: string;
  kind: string;
  description: string;
  languages: string[];
  license: string;
  license_url?: string;
  license_checked?: string;
  files: { size_bytes: number }[];
}

export const KIND_LABELS: Record<string, string> = {
  stt: "Speech to text",
  diarization: "Speaker identification",
  tts: "Text to speech",
  clone: "Voice cloning",
  text: "Language models",
};

/** The model catalog the app ships, read at build time so this page cannot drift from it. */
export function readCatalog(): CatalogModel[] {
  const data = JSON.parse(readRepoFile("catalog/models.json")) as { models: CatalogModel[] };
  return data.models;
}

export function modelSize(model: CatalogModel): string {
  const bytes = model.files.reduce((sum, file) => sum + file.size_bytes, 0);
  return bytes >= 1e9 ? `${(bytes / 1e9).toFixed(1)} GB` : `${Math.round(bytes / 1e6)} MB`;
}
