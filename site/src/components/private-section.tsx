import { SectionFrame } from "@/components/section-frame";

const FACTS = [
  {
    title: "Nothing is uploaded",
    body: "Audio and text are processed on your computer and never leave it.",
  },
  {
    title: "Downloads you start",
    body: "The only network use is fetching models, each checked against a published checksum.",
  },
  {
    title: "No account, no telemetry",
    body: "There is nothing to sign in to and nothing reports back.",
  },
  { title: "Open source", body: "MIT licensed, so you can read the code and check all of this." },
];

export function PrivateSection({ index }: { index: string }) {
  return (
    <SectionFrame id="private" index={index} name="Private by design" tag="Privacy">
      <div className="reveal p-6 md:p-10">
        <h2 className="max-w-xl text-2xl font-semibold tracking-tight sm:text-3xl">
          Your audio stays <span className="text-accent">yours</span>.
        </h2>
        <ul className="mt-8 grid gap-px overflow-hidden rounded-md border border-line bg-line sm:grid-cols-2 lg:grid-cols-4">
          {FACTS.map((fact) => (
            <li key={fact.title} className="bg-bg p-5">
              <h3 className="font-medium">{fact.title}</h3>
              <p className="mt-2 text-sm text-muted">{fact.body}</p>
            </li>
          ))}
        </ul>
      </div>
    </SectionFrame>
  );
}
