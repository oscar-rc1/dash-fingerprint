#!/bin/bash

if [ ! $# -eq 2 ]; then
	echo "Usage: watch_all.sh [interface] [base_url]"
	exit 1
fi

mkdir -p tests/auto

for video in videos/dash/*; do
	video_name=`basename "$video"`

	if [ ! -f "tests/auto/$video_name" ]; then
		cargo run --release -- network \
			--video "$2/$video_name/playlist.mpd" \
			"$1" \
			"tests/auto/$video_name"
	fi
done
