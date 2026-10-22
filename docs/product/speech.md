# Text to speech

`vtt speak` and the **Speak** screen read English text aloud with [Kokoro-82M](https://huggingface.co/hexgrad/Kokoro-82M).
Everything runs on this computer.

## How it works

1. The text is cut into sentences. A full stop after "Mr.", "Dr." or an initial does not end one, and a blank line
   ends a paragraph, which gets a longer pause.
2. Each sentence is normalised: numbers, years ("1984" is "nineteen eighty four"), ordinals, percentages, dollar
   amounts and common abbreviations are turned into words.
3. Each word is looked up in a pronunciation dictionary and turned into phoneme symbols. The dictionaries are the
   American English "gold" and "silver" lists of the [misaki](https://github.com/hexgrad/misaki) project (about
   180,000 words, Apache-2.0). Plurals, past tenses and `-ing` forms of known words are built with misaki's rules.
   A word that is not in the dictionaries is read as two known words joined, spelled out as letters if it is a short
   acronym, or guessed from its spelling.
4. The phonemes become numbers (the model's own alphabet) and the model speaks them in the chosen voice. A sentence
   that is too long for the model is cut again at a comma or a space.
5. The sentences are joined with pauses and written as a 16-bit mono WAV at 24 kHz.

Pronunciation is done in plain Rust, so the app needs no extra speech tools. The model runs on ONNX Runtime through
the `ort` crate, linked into the program. (The `tract` runtime used for speaker identification cannot run this model:
the length of its output depends on durations the model predicts while running.)

## The files

The catalog entry `kokoro-82m` (about 102 MB, all pinned to a commit with a SHA-256) installs the model
(`model_quantized.onnx` from [onnx-community/Kokoro-82M-v1.0-ONNX](https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX),
8-bit quantized: a quarter of the size and faster, still clearly understood), its `tokenizer.json`, the two misaki
dictionaries, and six voices: `af_heart`, `af_bella`, `af_nicole`, `af_sarah` (American women) and `am_adam`,
`am_michael` (American men). More voices are one file each (about 0.5 MB) and can be added to the catalog.

**Licences.** The Kokoro model and the ONNX export are Apache-2.0, as are the misaki dictionaries; ONNX Runtime and
`ort` are MIT and Apache-2.0. The voices are part of the Kokoro release under the same licence. The licence of the
text the voices were trained on was not reviewed (see Risks in the roadmap).

## Checked by listening with Whisper

`crates/core/tests/tts_e2e.rs` (ignored; it needs the models) speaks sentences and transcribes them with Whisper:

| Spoken                                                                                  | Whisper heard                                                             |
| --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| The quick brown fox jumps over the lazy dog.                                            | the quick brown fox jumps over the lazy dog                               |
| On March 3rd, 2024, we sold 1,250 tickets for 25 dollars each. That is 90% of the room! | on march 3rd 2024 we sold 1250 tickets for 25 each that is 90 of the room |
| Kokoro speaks one sentence at a time. It pauses between … paragraphs.                   | **kakara** speaks one sentence at a time it pauses between … paragraphs   |

Numbers, dates and two voices come through correctly. The one miss is a word that is not in the dictionary and is
guessed from its spelling. A machine listener is a weak judge of naturalness: how pleasant the voices sound has not
been measured.

## Why not Piper

The roadmap named Kokoro and Piper. Piper is not included because it cannot be done under this project's licence
rules yet:

- Piper turns text into phonemes with **espeak-ng, which is GPL-3**. Linking it would make the app GPL; running it as a
  separate program would need shipping and updating that program on three platforms.
- Piper voices each carry the licence of the recording they were trained on, and many are non-commercial or unclear.
  Each voice's model card would need review before it could be listed.

A Piper path would need a permissive phonemizer for each voice's language and a licence review per voice. Kokoro covers
the English case now; other languages are the reason to revisit this.

## Limits

- English only, American pronunciation (the British voices need the British dictionaries and are not shipped).
- No part-of-speech tagging, so words spelled alike but said differently ("read", "live") use their default sound.
- Names and rare words are guessed from their spelling.
- No SSML, no emphasis control, no pitch.
- Speech is not stored in the library yet; save it as a file.
