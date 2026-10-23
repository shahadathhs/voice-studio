# Text to speech

`vtt speak` and the **Speak** screen read text aloud, in eight languages, with [Kokoro-82M](https://huggingface.co/hexgrad/Kokoro-82M).
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
5. The sentences are joined with pauses and written as a 16-bit mono WAV at 24 kHz, or losslessly compressed to FLAC when asked.

Pronunciation is done in plain Rust, so the app needs no extra speech tools. The model runs on ONNX Runtime through
the `ort` crate, linked into the program. Speaker identification uses the same runtime, so the program carries one
inference engine. (An earlier version ran speaker identification on the pure-Rust `tract`, which cannot run this
model: the length of its output depends on durations the model predicts while running.)

## Languages

| Language                   | Voices | How the pronunciation is made                      | Checked by Whisper              |
| -------------------------- | ------ | -------------------------------------------------- | ------------------------------- |
| American English           | 6      | misaki dictionaries plus spelling rules            | exact                           |
| British English            | 8      | misaki British dictionaries                        | exact                           |
| Spanish (Castilian sounds) | 3      | rules; stress from accents and endings             | exact                           |
| Italian                    | 2      | rules plus a small list of respelled words         | within 0.05 to 0.20 (digits)    |
| Brazilian Portuguese       | 3      | rules (nasals, palatalised t/d, reduced vowels)    | within 0.10                     |
| French                     | 1      | rules plus about 90 irregular words                | within 0.15                     |
| Hindi                      | 4      | Devanagari rules with schwa deletion               | within 0.3 (aspirates are weak) |
| Mandarin Chinese           | 8      | pinyin word lists (MIT), tone changes, tone arrows | within 0.4 to 0.75              |

Each language is written in plain Rust (`crates/core/src/tts/<language>/`): spelling to sounds, stress, and numbers
read as words (decimal marks, thousands, "ventitré", "quatre-vingt-dix", Hindi's 0 to 99, Chinese 万/亿). **Chinese**
uses `pinyin-data` and `phrase-pinyin-data` (both MIT, 2.2 MB, downloaded with the voices): a character has its
most common reading, known words override it, 不 and 一 change tone, and a run of third tones rises. A model of
what a native speaker would find natural is missing: **no native speaker has listened to any of these**. The
check is `crates/core/tests/speech_*_e2e.rs`: each sentence is spoken, Whisper listens in that language, and the
share of differing letters must stay under a limit; it proves a recogniser understands the speech, not that it
sounds good. Known weaknesses are in each language's notes in the code: Italian has no open/closed vowels,
French has no liaison, Portuguese guesses open and closed e/o, Hindi cannot show voiced aspirates (Kokoro has no
sign for them), Chinese has no word segmentation beyond the lists.

**Japanese is not included.** Kokoro has Japanese voices, but reading kanji needs a large dictionary and a
morphological analyser. Kana alone would make real Japanese text unusable, so it waits for a dictionary decision.
**Bengali has no voice**: Kokoro has none and no permissively licensed alternative has been confirmed (see the
roadmap).

Text can be translated into the voice's language first with the language model (`vtt speak --translate-to es`, or the
checkbox on the Speak screen), so English text can be spoken in Spanish; quality follows the translation quality
(see [cli.md](cli.md)).

## The files

The catalog entry `kokoro-82m` (about 120 MB, all pinned to a commit with a SHA-256) installs the model
(`model_quantized.onnx` from [onnx-community/Kokoro-82M-v1.0-ONNX](https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX),
8-bit quantized: a quarter of the size and faster, still clearly understood), its `tokenizer.json`, the two misaki
dictionaries (American and British), the Chinese word lists, and 35 voices (one 0.5 MB file each; the voice
names start with a letter for the language and one for the gender, `ef_dora` is a Spanish woman).

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

- English, Spanish, French, Italian, Portuguese, Hindi and Chinese only; no Japanese or Bengali (see above).
- One voice speaks one language: a sentence in another language is read with that language's rules and sounds wrong.
- No part-of-speech tagging, so words spelled alike but said differently ("read", "live") use their default sound.
- Names and rare words are guessed from their spelling.
- No SSML, no emphasis control, no pitch.
- Speech is not stored in the library yet; save it as a file.
