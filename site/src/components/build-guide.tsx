import { CodeBlock } from "@/components/code-block";
import { BUILD_STEPS, PREREQUISITES } from "@/lib/build-steps";

export function BuildGuide() {
  return (
    <>
      <h2 className="mt-10 font-mono text-[11px] tracking-widest text-muted uppercase">You need</h2>
      <ul className="mt-3 space-y-2 text-sm text-muted">
        {PREREQUISITES.map((item) => (
          <li key={item} className="flex gap-3">
            <span className="mt-2 size-1.5 shrink-0 bg-accent" aria-hidden="true" />
            {item}
          </li>
        ))}
      </ul>
      <ol className="mt-10 space-y-10">
        {BUILD_STEPS.map((step, i) => (
          <li key={step.title}>
            <h2 className="font-mono text-[11px] tracking-widest text-muted uppercase">
              {String(i + 1).padStart(2, "0")} / {step.title}
            </h2>
            <p className="mt-3 max-w-2xl text-muted">{step.body}</p>
            {step.code ? (
              <div className="mt-4">
                <CodeBlock code={step.code} />
              </div>
            ) : null}
          </li>
        ))}
      </ol>
    </>
  );
}
