import { ButtonLink } from "@/components/button-link";
import { SectionFrame } from "@/components/section-frame";

export function GetSection({ index }: { index: string }) {
  return (
    <SectionFrame id="get" index={index} name="Get Voice Studio" tag="Free">
      <div className="reveal px-6 py-16 text-center md:px-10">
        <h2 className="text-3xl font-semibold tracking-tight sm:text-4xl">
          Build it <span className="text-accent">yourself</span>.
        </h2>
        <p className="mx-auto mt-4 max-w-xl text-muted">
          Voice Studio builds from source on macOS, Windows and Linux. A copy you build yourself
          starts normally, with no signing warnings.
        </p>
        <div className="mt-8">
          <ButtonLink href="/build">Build from source</ButtonLink>
        </div>
      </div>
    </SectionFrame>
  );
}
