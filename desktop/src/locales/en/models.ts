/** The Models screen. Model names and descriptions come from the catalog; descriptions can be translated here. */
export const models = {
  "models.title": "Models",
  "models.subtitle": "Downloaded once, then used offline. Remove any you don't need.",
  "models.installedCount": "{installed} of {total} installed",
  "models.onDisk": "{size} on disk",
  "models.none.title": "No models to show",
  "models.none.text": "Models are only available in the desktop app.",
  "models.storedIn": "Stored in {path}",
  "models.install": "Install",
  "models.installed": "Installed",
  "models.remove": "Remove",
  "models.downloading": "Downloading {name}",
  "models.downloaded": "{done} of {total}",
  "models.recommended": "Recommended",
  "models.licence": "{license} licence",
  "models.canTranslate": "can translate to English",
  "models.group.stt.title": "Transcription",
  "models.group.stt.purpose":
    "Turn audio and video into text. You need one of these; larger models are more accurate but slower. Not sure? Start with Whisper small.",
  "models.group.diarization.title": "Speaker labels",
  "models.group.diarization.purpose":
    "Optional. Adds who said what to a transcript. It works alongside a transcription model, and you switch it on with Identify speakers.",
  "models.group.tts.title": "Speech",
  "models.group.tts.purpose":
    "Read text aloud with natural voices. The Speak screen needs one of these.",
  "models.group.text.title": "Summaries and translation",
  "models.group.text.purpose":
    "Write summaries and translations of transcripts, on this computer. Optional; the larger model is clearer, especially outside English and Spanish, but needs more memory.",
  "models.goLibrary": "Go to Library",
  "models.goTranscribe": "Go to Transcribe",
  "models.goSpeak": "Go to Speak",
  "modelInfo.whisper-tiny":
    "Fastest and least accurate. For quick drafts of clear, close-up speech.",
  "modelInfo.whisper-base": "Fast and light. Good for clean recordings in a quiet room.",
  "modelInfo.whisper-small":
    "The best balance for most people: copes with accents and some background noise.",
  "modelInfo.whisper-medium": "High accuracy for difficult audio. Noticeably slower.",
  "modelInfo.whisper-large-v3-turbo":
    "Close to the best accuracy at a much higher speed. Cannot translate.",
  "modelInfo.whisper-large-v3-turbo-q5":
    "The turbo model compressed to about a third of the download, a little less precise.",
  "modelInfo.whisper-large-v3":
    "Highest accuracy and slowest. For hard recordings where quality matters most.",
  "modelInfo.speaker-identification":
    "Tells voices apart so a transcript can label who is speaking. Small and fast on the CPU.",
  "modelInfo.kokoro-82m":
    "Reads text aloud in natural voices: American and British English and Spanish, with more languages to come. Runs on the CPU.",
  "modelInfo.qwen2.5-1.5b":
    "Writes summaries and translations of transcripts in about 20 languages. Needs about 2 GB of memory. Read what it writes: it can make mistakes.",
  "modelInfo.qwen2.5-7b":
    "A larger model for summaries and translation: clearer and more accurate, especially in languages other than English and Spanish. Needs about 6 GB of memory and is slower.",
  "onboarding.title": "Choose what to download",
  "onboarding.lead":
    "Models run on this computer, so they need to be downloaded once. You can add or remove any of them later in Models.",
  "onboarding.total": "{size} to download",
  "onboarding.downloading": "Downloading… this screen closes when transcription is ready.",
  "onboarding.download": "Download",
  "onboarding.skip": "Skip for now",
  "onboarding.privacy":
    "Downloads come from the model authors' public hosting. No account, and nothing about you is sent.",
} as const;
