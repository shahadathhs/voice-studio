import { ImageResponse } from "next/og";
import { LOGO_BARS } from "@/lib/logo-bars";

export const size = { width: 180, height: 180 };
export const contentType = "image/png";

export default function AppleIcon() {
  return new ImageResponse(
    <div
      style={{
        width: "100%",
        height: "100%",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        gap: 8,
        background: "#1f6f5c",
      }}
    >
      {LOGO_BARS.map((h, i) => (
        <div key={i} style={{ width: 14, height: h * 5.6, borderRadius: 7, background: "#fff" }} />
      ))}
    </div>,
    size,
  );
}
