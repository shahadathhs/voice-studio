<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { theme, type ThemePreference } from "../lib/theme.svelte";
  import Segmented, { type SegmentOption } from "./Segmented.svelte";

  let { compact = false }: { compact?: boolean } = $props();

  const options = $derived<readonly SegmentOption<ThemePreference>[]>([
    { value: "system", label: t("theme.system"), icon: "monitor" },
    { value: "light", label: t("theme.light"), icon: "sun" },
    { value: "dark", label: t("theme.dark"), icon: "moon" },
  ]);
</script>

<Segmented
  {options}
  value={theme.preference}
  label={t("theme.label")}
  {compact}
  onchange={(value) => theme.set(value)}
/>
