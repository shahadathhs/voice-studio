import type { CatalogKey } from "../en";
import { assist } from "./assist";
import { clone } from "./clone";
import { common } from "./common";
import { dictation } from "./dictation";
import { library } from "./library";
import { models } from "./models";
import { record } from "./record";
import { speak } from "./speak";
import { transcribe } from "./transcribe";
import { voices } from "./voices";

/** Spanish. Typed against the English keys, so a missing or extra message fails the typecheck. */
export const es: Record<CatalogKey, string> = {
  ...assist,
  ...clone,
  ...common,
  ...dictation,
  ...transcribe,
  ...library,
  ...models,
  ...record,
  ...speak,
  ...voices,
};
