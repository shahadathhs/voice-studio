# Speaker identification

`vtt transcribe --diarize` and the **Identify speakers** option in the app label every line of a transcript
with who says it: `SPEAKER_00`, `SPEAKER_01`, … in order of first appearance. In the app a speaker can be
renamed in the viewer; renaming two speakers to the same name merges them.

## How it works

The method is the one of the earlier Python prototype (git tag `python-final`), with a different model:

1. Whisper transcribes the file as usual, giving lines with start and end times.
2. Each line of 3 s or more is cut into 1.5 s windows every 0.5 s; shorter lines of at least 1 s are taken whole.
   Each piece is trimmed to the part that contains speech, and pieces with under 0.8 s of speech are skipped,
   because one short word gives an unreliable voiceprint.
3. The speaker model turns each piece into a 256-number voiceprint (an _embedding_). Audio is converted to
   80-bin Kaldi-style log mel features first (`diarize::fbank`).
4. Voiceprints are clustered bottom-up (average linkage, cosine distance). Merging stops at `--speakers N`
   clusters, or when the closest two clusters are further apart than the threshold (default **0.45**).
5. A line takes the most common cluster among its pieces; lines without a piece take the label of the nearest
   piece in time. A line of 2 s or less that sits between two lines of the same other speaker is flipped to
   that speaker, which removes one-off flicker.

Everything runs on the CPU through ONNX Runtime (the same runtime as speech), with no extra setup. Embedding runs at roughly 20× real time,
so a one-hour recording takes a few minutes after transcription.

## The model

`wespeaker-resnet34`, the authors' own ONNX export from
[Wespeaker/wespeaker-voxceleb-resnet34](https://huggingface.co/Wespeaker/wespeaker-voxceleb-resnet34), pinned to
commit `ff1ac5b`, 26.5 MB, SHA-256 `9fea6516…` (see `catalog/models.json`). Install it with
`vtt models pull wespeaker-resnet34` or from the Models screen; `--diarize` offers to download it.

**Licence.** The model card says Apache-2.0. The WeSpeaker documentation adds that its VoxCeleb models "follow
the licence of the dataset", CC BY 4.0 (attribution). Both allow commercial use, but the training data's own terms
were not independently reviewed. Review this before the first public release (see Risks in the roadmap).

## Evaluation

`scripts/make-diarization-eval.sh` builds a labelled set from six macOS voices saying four sentences each; the
ignored test `crates/core/tests/diarize_e2e.rs` joins them into one conversation with a known speaker per line
and scores how many pairs of lines are grouped correctly (1.0 = perfect). Result with the shipped model:

| Threshold | Speakers found (6 real) | Pair accuracy |
| --------- | ----------------------- | ------------- |
| 0.3       | 11                      | 0.949         |
| 0.4       | 6                       | 1.000         |
| **0.45**  | **6**                   | **1.000**     |
| 0.5       | 6                       | 1.000         |
| 0.6       | 5                       | 0.942         |
| 0.7       | 4                       | 0.826         |
| 0.8       | 2                       | 0.594         |
| 0.9       | 1                       | 0.130         |
| 6 given   | 6                       | 1.000         |

The default sits in the middle of the range that finds exactly six. A second check transcribed a three-voice
conversation end to end with Whisper base: all eight lines were attributed correctly.

**What this does and does not show.** The voices are synthetic: clean, one speaker at a time, no overlap, no
background noise, and each voice is very consistent. Real recordings vary more within one speaker, so expect the
best threshold to be somewhat higher, and expect worse results with overlapping speech, similar voices, noise or
many speakers. When you know how many people speak, `--speakers N` is more reliable than the threshold. A set of
real, labelled recordings is still needed before this is called measured.

## A real recording

Checked on a 3-minute NASA recording (Apollo 13 air-to-ground radio, public domain: three voices, static, radio
clipping). Whisper small transcribed it well, but the default threshold found **33 speakers**. Sweeping the
threshold: 0.6 gives 8, 0.7 gives 3 (the right number), 0.8 gives 1. The merge distances rise smoothly with no clear
gap, so choosing the count automatically from their shape was tried and does not work on this audio. Noisy
audio needs a much higher threshold than the clean synthetic set, so a single default cannot suit both:
**when you know the number of speakers, pass `--speakers N`**; otherwise raise `--diarize-threshold` for noisy
recordings.

## Limits

- Overlapping speech is not handled: a line gets one speaker.
- A line is one speaker, even if Whisper's line spans a change of speaker.
- There is no progress report during labelling yet; the job shows full progress while it finishes.
