#!/bin/sh
# Builds the small labelled set used to evaluate speaker identification: six macOS voices, four
# sentences each, as 16 kHz mono WAV files named <voice>-<n>.wav in the directory given (macOS only).
#
#   scripts/make-diarization-eval.sh /tmp/diarization-eval
#   VTT_TEST_SPEAKER_MODEL=<voxceleb_resnet34.onnx> VTT_TEST_SPEAKER_AUDIO=/tmp/diarization-eval \
#     cargo test -p vtt-core --release --test diarize_e2e -- --ignored --nocapture

set -eu

dir="${1:?usage: make-diarization-eval.sh <output directory>}"
mkdir -p "$dir"

n=0
for voice in Daniel Karen Moira Rishi Samantha Tessa; do
  n=0
  while IFS= read -r sentence; do
    n=$((n + 1))
    say -v "$voice" -o "$dir/$voice-$n.aiff" "$sentence"
    afconvert -f WAVE -d LEI16@16000 -c 1 "$dir/$voice-$n.aiff" "$dir/$voice-$n.wav"
    rm "$dir/$voice-$n.aiff"
  done <<'SENTENCES'
We should ship the importer behind a flag first, so support can turn it on for each customer.
The quarterly numbers look better than expected, but the cost of hosting keeps growing every month.
I think the pricing page needs clearer wording before we announce anything to the wider public.
Let us meet again on Thursday afternoon and decide which of these tasks can wait until next week.
SENTENCES
done
echo "wrote $(ls "$dir"/*.wav | wc -l | tr -d ' ') files to $dir"
