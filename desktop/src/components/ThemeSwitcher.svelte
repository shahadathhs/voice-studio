<script lang="ts">
  import { theme, type ThemePreference } from "../lib/theme.svelte";
  import Segmented, { type SegmentOption } from "./Segmented.svelte";

  let { compact = false }: { compact?: boolean } = $props();

  const OPTIONS: readonly SegmentOption<ThemePreference>[] = [
    { value: "system", label: "System", icon: "monitor" },
    { value: "light", label: "Light", icon: "sun" },
    { value: "dark", label: "Dark", icon: "moon" },
  ];
</script>

<Segmented
  options={OPTIONS}
  value={theme.preference}
  label="Appearance"
  {compact}
  onchange={(value) => theme.set(value)}
/>
