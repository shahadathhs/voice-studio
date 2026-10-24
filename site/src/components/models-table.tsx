import { KIND_LABELS, modelSize, type CatalogModel } from "@/lib/models-data";

export function ModelsTable({ kind, models }: { kind: string; models: CatalogModel[] }) {
  return (
    <section className="mt-10">
      <h2 className="text-xl font-semibold tracking-tight">{KIND_LABELS[kind] ?? kind}</h2>
      <div className="mt-4 overflow-x-auto rounded-md border border-line">
        <table className="w-full border-collapse text-left text-sm">
          <thead className="bg-raised font-mono text-[11px] tracking-widest text-muted uppercase">
            <tr>
              <th className="p-3">Model</th>
              <th className="p-3">Size</th>
              <th className="p-3">Languages</th>
              <th className="p-3">Licence</th>
            </tr>
          </thead>
          <tbody>
            {models.map((model) => (
              <tr key={model.id} className="border-t border-line align-top">
                <td className="p-3">
                  <p className="font-medium">{model.name}</p>
                  <p className="mt-1 max-w-md text-muted">{model.description}</p>
                  <p className="mt-1 font-mono text-xs text-muted">{model.id}</p>
                </td>
                <td className="p-3 whitespace-nowrap">{modelSize(model)}</td>
                <td className="p-3">{model.languages.join(", ")}</td>
                <td className="p-3">
                  {model.license_url ? (
                    <a className="underline underline-offset-4" href={model.license_url}>
                      {model.license}
                    </a>
                  ) : (
                    model.license
                  )}
                  {model.license_checked ? (
                    <p className="mt-1 text-xs text-muted">checked {model.license_checked}</p>
                  ) : null}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}
