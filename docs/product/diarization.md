# Speaker identification

`vtt transcribe --diarize` and the **Identify speakers** option in the app label every line of a transcript
with who says it: `SPEAKER_00`, `SPEAKER_01`, … in order of first appearance. In the app a speaker can be
renamed in the viewer; renaming two speakers to the same name merges them.

## How it works

Two small models and a clustering step, all run on the CPU through ONNX Runtime (the same runtime as speech):

1. Whisper transcribes the file as usual, giving lines with start and end times.
2. **Segmentation.** A segmentation model (pyannote segmentation-3.0) slides over the recording in 10 s windows,
   a new one every 2.5 s, so each moment is seen four times. For every 10 ms it says which of up to three
   _local_ speakers is talking, overlaps included.
3. **Voiceprints.** What one local speaker says in a window is cut out and turned by the speaker model (WeSpeaker
   ResNet34) into a 256-number voiceprint, using 80-bin log mel features (`diarize::fbank`). A local speaker with
   under about a second of speech is skipped: one short word gives an unreliable voiceprint. Lines the
   segmentation hardly covered are embedded whole.
4. **Clustering.** The voiceprints of the whole recording are clustered bottom-up (average linkage, cosine
   distance). Voiceprints made from under 2 s of speech never start a speaker; they join the nearest cluster.
5. **Voting.** Every cluster votes, per 10 ms, for the stretches it covers, and a line takes the speaker with the
   most votes during it. Because the timeline is built from the segmentation, overlapping speech is heard; a
   line still gets one label, the one who talks most.

**How many speakers.** `--speakers N` stops merging at exactly N clusters. Without it, the number is found from
the merge distances: the voices of different people are far apart, so the last few merges show one big jump,
and the cut is placed before it. If there is no clear jump (noisy audio where voiceprints do not separate the
voices) the cut falls back to a distance of 0.55, which prefers merging to inventing speakers.
`--diarize-threshold X` replaces all of this with a fixed distance.

A one-hour recording takes a few minutes after transcription.

## The models

One catalog entry, `speaker-identification`, installs both files (about 32.5 MB). Install it with
`vtt models pull speaker-identification` or from the Models screen; `--diarize` offers to download it.

| File                     | Model                                                                                                                            | Size    | Licence                |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------- | ------- | ---------------------- |
| `voxceleb_resnet34.onnx` | [Wespeaker/wespeaker-voxceleb-resnet34](https://huggingface.co/Wespeaker/wespeaker-voxceleb-resnet34), commit `ff1ac5b`          | 26.5 MB | Apache-2.0 (see below) |
| `segmentation-3.0.onnx`  | [pyannote segmentation-3.0](https://huggingface.co/csukuangfj/sherpa-onnx-pyannote-segmentation-3-0), ONNX export by sherpa-onnx | 6.0 MB  | MIT                    |

Both are pinned to a commit and checked against a SHA-256 (see `catalog/models.json`).

**Licence.** The WeSpeaker model card says Apache-2.0, and its documentation adds that its VoxCeleb models "follow
the licence of the dataset", CC BY 4.0 (attribution). Both allow commercial use, but the training data's own terms
were not independently reviewed. The pyannote segmentation model is MIT; the licence of its training data was not
reviewed either. Review both before the first public release (see Risks in the roadmap).

## Evaluation

`scripts/make-diarization-eval.sh` builds a labelled set from six macOS voices saying four sentences each; the
ignored test `crates/core/tests/diarize_e2e.rs` joins them into one conversation with a known speaker per line
and scores how many pairs of lines are grouped correctly (1.0 = perfect):

| Setting           | Speakers found (6 real) | Pair accuracy |
| ----------------- | ----------------------- | ------------- |
| threshold 0.2–0.4 | 6                       | 1.000         |
| threshold 0.5     | 5                       | 0.942         |
| threshold 0.6     | 5                       | 0.942         |
| threshold 0.7     | 4                       | 0.826         |
| threshold 0.8     | 2                       | 0.594         |
| **automatic**     | **6**                   | **1.000**     |
| 6 given           | 6                       | 1.000         |

`scripts/degrade-diarization-eval.sh` turns the same set into a radio-like one (band-limited, clipped, noisy,
every voice through the same bad channel) for `crates/core/tests/diarize_real_e2e.rs`.

**A real recording.** A 3-minute NASA recording (Apollo 13 air-to-ground radio, public domain: three voices,
static, radio clipping). The first version of this feature (1.5 s windows and a fixed threshold of 0.45) found
**33 speakers** in it. The current automatic setting finds **3**.

**What this does and does not show.** The synthetic voices are clean, one at a time, and very consistent, and the
radio recording has no labels, so neither measures accuracy on real meetings (similar voices, crosstalk, many
speakers). When you know how many people speak, `--speakers N` is more reliable than the automatic count. A set
of real, labelled recordings is still needed before this is called measured.

## Limits

- A line is one speaker, even if Whisper's line spans a change of speaker; overlapping speech is heard but only
  the speaker who talks most gets the line.
- At most three people speaking at once in a 10 s window (a limit of the segmentation model).
- There is no progress report during labelling yet; the job shows full progress while it finishes.
