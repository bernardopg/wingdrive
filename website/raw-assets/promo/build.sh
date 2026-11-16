#!/usr/bin/env bash
# Renders one cut end to end: video frames, score, loudness-normalized mix, MP4.
#   ./build.sh <youtube|github|shorts> [workers]
set -euo pipefail
cd "$(dirname "$0")"
cut=$1; workers=${2:-4}
out=out/final; exp=out/export; mkdir -p "$out" "$exp"
./prepare.sh
bun render.mjs "$cut" --fps 60 --sub 4 --workers "$workers" --out "$out"
.venv/bin/python audio.py "$out/$cut.cues.json" "$out/$cut-audio.wav"
# Two-pass loudnorm to -14 LUFS / -1.5 dBTP (YouTube's reference level).
m=$(ffmpeg -hide_banner -i "$out/$cut-audio.wav" -af loudnorm=I=-14:TP=-1.5:LRA=11:print_format=json -f null - 2>&1 | sed -n '/^{/,/^}/p')
g() { echo "$m" | python3 -c "import json,sys;print(json.load(sys.stdin)['$1'])"; }
ln="loudnorm=I=-14:TP=-1.5:LRA=11:measured_I=$(g input_i):measured_TP=$(g input_tp):measured_LRA=$(g input_lra):measured_thresh=$(g input_thresh):offset=$(g target_offset):linear=true"
name=wingdrive-$cut
ffmpeg -y -loglevel error -i "$out/$cut-video.mkv" -i "$out/$cut-audio.wav" -map 0:v -map 1:a \
	-af "$ln,aresample=48000" -c:a aac -b:a 320k \
	-vf "format=yuv420p" -c:v libx264 -preset slow -crf 15 -profile:v high -pix_fmt yuv420p \
	-color_primaries bt709 -color_trc bt709 -colorspace bt709 -movflags +faststart -shortest "$exp/$name.mp4"
if [ "$cut" = github ]; then
	# A copy under 10 MB, the limit for videos dropped into a GitHub README.
	dur=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$exp/$name.mp4")
	vb=$(python3 -c "print(int((9.3*8*1024*1024/$dur - 160000)))")
	ffmpeg -y -loglevel error -i "$exp/$name.mp4" -c:v libx264 -preset slow -b:v "$vb" -pass 1 -passlogfile "$out/x264" -an -f mp4 /dev/null
	ffmpeg -y -loglevel error -i "$exp/$name.mp4" -c:v libx264 -preset slow -b:v "$vb" -pass 2 -passlogfile "$out/x264" -c:a aac -b:a 160k -movflags +faststart "$exp/$name-readme.mp4"
fi
ls -la $exp/
