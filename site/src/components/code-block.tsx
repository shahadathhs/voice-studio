export function CodeBlock({ code }: { code: string }) {
  return (
    <pre className="overflow-x-auto rounded-md border border-line-strong bg-sunken p-4 font-mono text-[13px] leading-relaxed">
      <code>{code}</code>
    </pre>
  );
}
