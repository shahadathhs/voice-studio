export const CLI_EXAMPLES = `vtt models pull whisper-small                  # download a model (resumable, checksum-verified)
vtt transcribe talk.m4a                        # print the text
vtt transcribe talk.m4a -f srt -o talk.srt     # subtitles; also txt, vtt, md, json
vtt transcribe meeting.mp4 --diarize           # label the speakers
vtt speak "Hello from my own computer" -o hello.wav
vtt library search "roadmap"                   # search everything you have transcribed
vtt --json transcribe talk.m4a                 # machine-readable events for scripts`;
