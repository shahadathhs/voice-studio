export interface CliGroup {
  title: string;
  code: string;
}

/** Keep in step with docs/product/cli.md. */
export const CLI_GROUPS: CliGroup[] = [
  {
    title: "Transcribe",
    code: `vtt models pull whisper-small                  # download a model (resumable, checksum-verified)
vtt transcribe talk.m4a                        # print the text
vtt transcribe talk.m4a -f srt -o talk.srt     # subtitles; also txt, vtt, md, json
vtt transcribe meeting.mp4 --diarize           # label the speakers`,
  },
  {
    title: "Record",
    code: `vtt devices                                    # list microphones
vtt record -o note.wav --live --transcribe     # live captions, then transcribe`,
  },
  {
    title: "Speak and voices",
    code: `vtt speak "Hello from my own computer" -o hello.wav
vtt speak "Hello" --translate-to es -o hola.wav
vtt voice create Warm --blend af_heart:0.6,af_bella:0.4
vtt voice clone Anna --from anna.wav --i-have-permission`,
  },
  {
    title: "Library, summaries, translation",
    code: `vtt library search "roadmap"
vtt summarize 12 --chapters --actions
vtt translate 12 --to zh -f srt -o talk.zh.srt`,
  },
  {
    title: "For scripts",
    code: `vtt --json transcribe talk.m4a                 # machine-readable events
vtt completions zsh                            # shell completions
vtt doctor                                     # GPU, devices, models, permissions`,
  },
];

export const EXIT_CODES: [string, string][] = [
  ["0", "Success"],
  ["2", "Usage error (bad flags)"],
  ["3", "Input error (file missing, unreadable or unsupported)"],
  ["4", "A required model is not installed"],
  ["6", "Engine failure (decode, inference, device)"],
  ["7", "Download failed"],
  ["130", "Cancelled (Ctrl-C)"],
];
