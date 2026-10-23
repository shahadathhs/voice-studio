<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { formatClock } from "../lib/format";
  import { speech } from "../lib/speech.svelte";
  import Icon from "./Icon.svelte";
</script>

<div class="player">
  <button
    class="play"
    aria-label={speech.playing ? t("player.pause") : t("player.play")}
    onclick={() => speech.togglePlay()}
  >
    <Icon name={speech.playing ? "pause" : "play"} />
  </button>
  <span class="mono time">{formatClock(speech.positionMs)}</span>
  <input
    class="seek"
    type="range"
    min="0"
    max={speech.durationMs}
    step="100"
    value={speech.positionMs}
    aria-label={t("speak.position")}
    oninput={(event) => speech.seek(Number(event.currentTarget.value))}
  />
  <span class="mono time muted">{formatClock(speech.durationMs)}</span>
  <button class="btn" onclick={() => speech.save()}><Icon name="down" />{t("speak.save")}</button>
</div>

<style>
  .player {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 16px;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--surface);
  }

  .play {
    display: grid;
    place-items: center;
    flex: none;
    width: 38px;
    height: 38px;
    border-radius: 8px;
    background: var(--accent);
    color: var(--accent-fg);
  }

  .seek {
    flex: 1;
    accent-color: var(--accent);
  }

  .time {
    min-width: 40px;
  }
</style>
