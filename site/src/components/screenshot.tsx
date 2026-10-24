interface ScreenshotProps {
  name: string;
  alt: string;
  width?: number;
  height?: number;
}

/** A real app screenshot in the visitor's theme (light by default, dark when the system is dark). */
export function Screenshot({ name, alt, width = 1440, height = 900 }: ScreenshotProps) {
  return (
    <picture>
      <source media="(prefers-color-scheme: dark)" srcSet={`/screenshots/${name}-dark.png`} />
      {/* eslint-disable-next-line @next/next/no-img-element */}
      <img
        src={`/screenshots/${name}-light.png`}
        alt={alt}
        width={width}
        height={height}
        loading="lazy"
        className="h-auto w-full rounded-md border border-line"
      />
    </picture>
  );
}
