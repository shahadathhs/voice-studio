import { common } from "./common";
import { library } from "./library";
import { models } from "./models";
import { speak } from "./speak";
import { transcribe } from "./transcribe";

/** English, the reference catalog: other languages must define exactly these keys. */
export const en = { ...common, ...transcribe, ...library, ...models, ...speak } as const;

/** Every key in a catalog, including plural forms such as `batch.ready.one`. */
export type CatalogKey = keyof typeof en;

type WithoutPlural<K> = K extends `${infer Base}.${"one" | "other"}` ? Base : K;

/** The keys you pass to `t()`: plural forms are chosen from `count`, so they appear without suffix. */
export type MessageKey = WithoutPlural<CatalogKey>;
