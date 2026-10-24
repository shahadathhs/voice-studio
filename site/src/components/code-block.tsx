export function CodeBlock({ code }: { code: string }) {
  return (
    <pre className="overflow-x-auto rounded-md border border-line bg-surface p-4 font-mono text-sm leading-relaxed">
      <code>{code}</code>
    </pre>
  );
}
