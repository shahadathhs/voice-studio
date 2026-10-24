interface ScreenshotProps {
  name: string;
  alt: string;
  /** Pixel size of the PNG, so the page reserves the space before it loads. */
  width: number;
  height: number;
  priority?: boolean;
}

/** A real app screenshot in the current theme: both are in the page and CSS shows the matching one. */
export function Screenshot({ name, alt, width, height, priority }: ScreenshotProps) {
  const common = {
    width,
    height,
    alt,
    loading: priority ? ("eager" as const) : ("lazy" as const),
    className: "h-auto w-full",
  };
  return (
    <>
      {/* eslint-disable-next-line @next/next/no-img-element */}
      <img
        src={`/screenshots/${name}-light.png`}
        {...common}
        className={`shot-light ${common.className}`}
      />
      {/* eslint-disable-next-line @next/next/no-img-element */}
      <img
        src={`/screenshots/${name}-dark.png`}
        {...common}
        className={`shot-dark ${common.className}`}
      />
    </>
  );
}
