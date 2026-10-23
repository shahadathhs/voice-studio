import { en, type CatalogKey, type MessageKey } from "../locales/en";
import { es } from "../locales/es";

/** A language the interface is translated into. Add one by adding a catalog in `locales/`. */
export interface Locale {
  code: string;
  /** The language's own name, so people can find it whatever language is showing. */
  name: string;
  messages: Readonly<Record<CatalogKey, string>>;
}

export const LOCALES: readonly Locale[] = [
  { code: "en", name: "English", messages: en },
  { code: "es", name: "Español", messages: es },
];

/** "system", or the code of one of `LOCALES`. */
export type LocalePreference = string;

const STORAGE_KEY = "voice-studio-locale";
const FALLBACK = LOCALES[0];

function find(code: string | null): Locale | undefined {
  return LOCALES.find((locale) => locale.code === code);
}

function readStored(): LocalePreference {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return find(stored) ? (stored ?? "system") : "system";
  } catch {
    return "system";
  }
}

/** The first system language we have a translation for, matching on the language part ("es-MX" → es). */
function fromSystem(): Locale {
  for (const tag of navigator.languages) {
    const match = find(tag.split("-")[0].toLowerCase());
    if (match) return match;
  }
  return FALLBACK;
}

type Params = Record<string, string | number>;

class I18n {
  preference = $state<LocalePreference>(readStored());
  readonly locale = $derived(find(this.preference) ?? fromSystem());

  constructor() {
    $effect.root(() => {
      $effect(() => {
        document.documentElement.lang = this.locale.code;
      });
    });
  }

  set(preference: LocalePreference): void {
    this.preference = preference;
    try {
      localStorage.setItem(STORAGE_KEY, preference);
    } catch {
      // The choice still applies for this session if storage is unavailable.
    }
  }

  /**
   * The message for `key` with `{name}` placeholders filled in. When `count` is given, the
   * plural form is chosen with `Intl.PluralRules`: `key.one`, `key.other` and so on.
   */
  t(key: MessageKey, params: Params = {}): string {
    return fill(this.#lookup(key, params), params);
  }

  /** Like `t`, but for text that may not be translated (a model's description): `fallback` if no message. */
  tOr(key: string, fallback: string): string {
    const messages: Readonly<Record<string, string>> = this.locale.messages;
    return messages[key] ?? fallback;
  }

  #lookup(key: MessageKey, params: Params): string {
    const messages: Readonly<Record<string, string>> = this.locale.messages;
    const english: Readonly<Record<string, string>> = FALLBACK.messages;
    const count = params.count;
    if (typeof count === "number") {
      const form = new Intl.PluralRules(this.locale.code).select(count);
      const plural = messages[`${key}.${form}`] ?? messages[`${key}.other`];
      if (plural) return plural;
    }
    return messages[key] ?? english[key] ?? key;
  }
}

function fill(message: string, params: Params): string {
  return message.replace(/\{(\w+)\}/g, (whole, name: string) => String(params[name] ?? whole));
}

export const i18n = new I18n();

/** Shorthand used everywhere in the interface: `t("nav.library")`. */
export const t = (key: MessageKey, params?: Params): string => i18n.t(key, params);
