import { SectionFrame } from "@/components/section-frame";
import { Screenshot } from "@/components/screenshot";
import type { ScreenInfo } from "@/lib/screens";

interface ScreenSectionProps {
  screen: ScreenInfo;
  index: string;
  flip: boolean;
}

/** One screen of the app: the pitch on one side, the real screenshot on the other. */
export function ScreenSection({ screen, index, flip }: ScreenSectionProps) {
  return (
    <SectionFrame id={screen.id} index={index} name={screen.name} tag="Screen">
      <div className="reveal grid gap-8 p-6 md:grid-cols-5 md:items-center md:gap-10 md:p-10">
        <div className={`md:col-span-2 ${flip ? "md:order-2" : ""}`}>
          <h2 className="text-2xl font-semibold tracking-tight sm:text-3xl">{screen.title}</h2>
          <p className="mt-4 text-muted">{screen.body}</p>
          <ul className="mt-5 space-y-2 text-sm text-muted">
            {screen.points.map((point) => (
              <li key={point} className="flex gap-3">
                <span className="mt-2 size-1.5 shrink-0 bg-accent" aria-hidden="true" />
                {point}
              </li>
            ))}
          </ul>
        </div>
        <div
          className={`overflow-hidden rounded-md border border-line-strong md:col-span-3 ${flip ? "md:order-1" : ""}`}
        >
          <Screenshot
            name={screen.shot}
            alt={screen.alt}
            width={screen.width}
            height={screen.height}
          />
        </div>
      </div>
    </SectionFrame>
  );
}
