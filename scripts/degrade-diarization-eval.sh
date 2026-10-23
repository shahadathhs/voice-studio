#!/bin/sh
# Turns the clean evaluation set (make-diarization-eval.sh) into a radio-like one: every clip is
# band-limited to 300-3000 Hz, clipped and mixed with white noise, like a noisy air-to-ground link.
# All voices share the one bad channel, which is what makes such recordings hard.
#
#   scripts/degrade-diarization-eval.sh /tmp/diarization-eval /tmp/diarization-eval-radio

set -eu

from="${1:?usage: degrade-diarization-eval.sh <clean set> <output directory>}"
to="${2:?usage: degrade-diarization-eval.sh <clean set> <output directory>}"
mkdir -p "$to"
for clip in "$from"/*.wav; do
  ffmpeg -y -loglevel error -i "$clip" -f lavfi -i "anoisesrc=a=0.04:c=white:r=16000" \
    -filter_complex "[0]highpass=f=300,lowpass=f=3000,volume=4,alimiter=limit=0.6[v];[v][1]amix=inputs=2:duration=first:normalize=0" \
    -ac 1 -ar 16000 "$to/$(basename "$clip")"
done
echo "wrote $(ls "$to"/*.wav | wc -l | tr -d ' ') files to $to"
