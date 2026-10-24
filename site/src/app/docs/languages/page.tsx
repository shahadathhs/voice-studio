import { Breadcrumbs } from "@/components/breadcrumbs";
import { DocsShell } from "@/components/docs-shell";
import {
  CLONING_LANGUAGES,
  INTERFACE_LANGUAGES,
  SPEECH_LANGUAGES,
  TRANSLATION_LANGUAGES,
} from "@/lib/languages";
import { pageMetadata } from "@/lib/seo";

export const metadata = pageMetadata({
  title: "Languages",
  description:
    "Languages Voice Studio supports for transcription, speech, voice cloning, translation and the interface, with honest notes on quality.",
  path: "/docs/languages",
});

function Chips({ items }: { items: string[] }) {
  return (
    <ul className="mt-3 flex flex-wrap gap-2 text-sm">
      {items.map((item) => (
        <li key={item} className="rounded-md border border-line px-2.5 py-1 text-muted">
          {item}
        </li>
      ))}
    </ul>
  );
}

function SpeechTable() {
  return (
    <div className="mt-4 overflow-x-auto rounded-md border border-line">
      <table className="w-full min-w-[28rem] border-collapse text-left text-sm">
        <thead className="bg-raised font-mono text-[11px] tracking-widest text-muted uppercase">
          <tr>
            <th className="p-3">Language</th>
            <th className="p-3">Voices</th>
            <th className="p-3">Machine check</th>
          </tr>
        </thead>
        <tbody>
          {SPEECH_LANGUAGES.map((language) => (
            <tr key={language.name} className="border-t border-line">
              <td className="p-3">{language.name}</td>
              <td className="p-3">{language.voices}</td>
              <td className="p-3 text-muted">{language.check}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default function LanguagesPage() {
  return (
    <DocsShell current="languages">
      <Breadcrumbs trail={[{ name: "Docs", href: "/docs" }, { name: "Languages" }]} />
      <h1 className="mt-4 text-3xl font-semibold tracking-tight sm:text-4xl">Languages</h1>
      <p className="mt-4 max-w-2xl text-muted">
        What works in which language. No native speaker has yet reviewed the speech or the
        translations, so treat quality in every language other than English as unreviewed.
      </p>
      <h2 className="mt-10 text-xl font-semibold tracking-tight">Transcription</h2>
      <p className="mt-3 max-w-2xl text-sm text-muted">
        The Whisper models are multilingual: let the language be detected, or choose one.
      </p>
      <h2 className="mt-10 text-xl font-semibold tracking-tight">Speech (35 voices)</h2>
      <SpeechTable />
      <p className="mt-3 max-w-2xl text-sm text-muted">
        The check is a speech recogniser listening to the output: it shows the words are understood,
        not that they sound natural. Japanese and Bengali speech are not available.
      </p>
      <h2 className="mt-10 text-xl font-semibold tracking-tight">Voice cloning</h2>
      <Chips items={CLONING_LANGUAGES} />
      <h2 className="mt-10 text-xl font-semibold tracking-tight">Summaries and translation (20)</h2>
      <Chips items={TRANSLATION_LANGUAGES} />
      <p className="mt-3 max-w-2xl text-sm text-muted">
        Written by a small local model, so output can contain errors. Spanish and Chinese were
        readable in testing; Bengali is unreliable and is flagged in the app.
      </p>
      <h2 className="mt-10 text-xl font-semibold tracking-tight">Interface</h2>
      <Chips items={INTERFACE_LANGUAGES} />
    </DocsShell>
  );
}
