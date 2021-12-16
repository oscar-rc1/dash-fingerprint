#!/bin/bash

if [ ! $# -eq 3 ]; then
	echo "Usage: watch_all.sh [interface] [base_url] [output_dir]"
	exit 1
fi

mkdir -p "$3"

for video in videos/dash/*; do
	video_name=`basename "$video"`

	if [ ! -f "$3/$video_name" ]; then
		cargo run --release -- network \
			--video "$2/$video_name/playlist.mpd" \
			"$1" \
			"$3/$video_name"
	fi
done
