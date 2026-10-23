import type { CatalogKey } from "../en";
import { assist } from "./assist";
import { common } from "./common";
import { library } from "./library";
import { models } from "./models";
import { speak } from "./speak";
import { transcribe } from "./transcribe";

/** Spanish. Typed against the English keys, so a missing or extra message fails the typecheck. */
export const es: Record<CatalogKey, string> = {
  ...assist,
  ...common,
  ...transcribe,
  ...library,
  ...models,
  ...speak,
};
