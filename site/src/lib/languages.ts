/** Keep in step with docs/product/speech.md and docs/product/cli.md. */
export interface SpeechLanguage {
  name: string;
  voices: number;
  check: string;
}

export const SPEECH_LANGUAGES: SpeechLanguage[] = [
  { name: "American English", voices: 6, check: "exact" },
  { name: "British English", voices: 8, check: "exact" },
  { name: "Spanish", voices: 3, check: "exact" },
  { name: "Italian", voices: 2, check: "close (digits differ a little)" },
  { name: "Brazilian Portuguese", voices: 3, check: "close" },
  { name: "French", voices: 1, check: "close" },
  { name: "Hindi", voices: 4, check: "close (aspirates are weak)" },
  { name: "Mandarin Chinese", voices: 8, check: "looser" },
];

export const TRANSLATION_LANGUAGES = [
  "English",
  "Spanish",
  "French",
  "German",
  "Italian",
  "Portuguese",
  "Dutch",
  "Russian",
  "Polish",
  "Turkish",
  "Arabic",
  "Hindi",
  "Bengali",
  "Urdu",
  "Chinese",
  "Japanese",
  "Korean",
  "Vietnamese",
  "Indonesian",
  "Thai",
];

export const CLONING_LANGUAGES = ["English", "Spanish", "French", "Italian", "Portuguese", "Hindi"];

export const INTERFACE_LANGUAGES = ["English", "Spanish"];
