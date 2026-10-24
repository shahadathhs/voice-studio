import { ImageResponse } from "next/og";
import { LOGO_BARS } from "@/lib/logo-bars";

export const alt = "Voice Studio: speech to text and back, offline";
export const size = { width: 1200, height: 630 };
export const contentType = "image/png";

export default function OpenGraphImage() {
  return new ImageResponse(
    <div
      style={{
        width: "100%",
        height: "100%",
        display: "flex",
        flexDirection: "column",
        justifyContent: "center",
        padding: 80,
        background: "#171716",
        color: "#ecebe7",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 6, height: 120 }}>
        {LOGO_BARS.map((h, i) => (
          <div
            key={i}
            style={{ width: 14, height: h * 5, borderRadius: 7, background: "#4db89b" }}
          />
        ))}
      </div>
      <div style={{ fontSize: 84, fontWeight: 700, marginTop: 40, letterSpacing: -2 }}>
        Voice Studio
      </div>
      <div style={{ fontSize: 40, marginTop: 16, color: "#a5a29b" }}>
        Speech to text and back, offline.
      </div>
      <div style={{ fontSize: 28, marginTop: 40, color: "#4db89b" }}>
        Open source · Windows · macOS · Linux
      </div>
    </div>,
    size,
  );
}
