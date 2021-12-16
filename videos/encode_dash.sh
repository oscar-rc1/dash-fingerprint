#!/bin/bash

RESOLUTIONS=(480 720 1080)
BITRATES=(2500 5000 8000)

set -e

encode_video() {
	local video=$1
	local video_name=`basename "$video" | sed -E 's/.+\[([^]]+)\]\.[a-z]+/\1/g'`

	if [ ! -f "$video" ]; then
		echo "ERROR: No such video: $video"
		return 1
	fi

	local framerate=$(
		ffprobe \
			-v error \
			-select_streams v \
			-of default=noprint_wrappers=1:nokey=1 \
			-show_entries stream=r_frame_rate \
			"$video" \
		| bc
	)

	local src_width=$(
		ffprobe \
			-v error \
			-select_streams v \
			-of default=noprint_wrappers=1:nokey=1 \
			-show_entries stream=width \
			"$video"
	)

	local src_height=$(
		ffprobe \
			-v error \
			-select_streams v \
			-of default=noprint_wrappers=1:nokey=1 \
			-show_entries stream=height \
			"$video"
	)

	mkdir -p "encoded/$video_name"

	if [ ! -f "encoded/$video_name/audio.m4a" ]; then
		ffmpeg \
			-i "$video" \
			-map 0:a:0 \
			-c:a aac \
			-b:a 128k \
			-map_chapters -1 \
			-y \
			"encoded/$video_name/audio.m4a"
	fi

	local dash_sources=("encoded/$video_name/audio.m4a#audio:id=aac")

	for i in {0..2}; do
		dash_sources+=("encoded/$video_name/video_${RESOLUTIONS[$i]}.mp4#video:id=${RESOLUTIONS[$i]}p")

		if [ ! -f "encoded/$video_name/video_${RESOLUTIONS[$i]}.mp4" ]; then
			local max_height=${RESOLUTIONS[$i]}
			local max_width=$(( ((($max_height*16) / 9) / 2) * 2 ))

			local height=$(( ((($max_width*$src_height) / $src_width) / 2) * 2 ))
			local width=$(( ((($src_width*$max_height) / $src_height) / 2) * 2 ))

			if (( $width > $max_width )); then
				width=$max_width
			else
				height=$max_height
			fi

			ffmpeg \
				-i "$video" \
				-map 0:v:0 \
				-c:v libx264 \
				-profile:v high \
				-level:v 4.0 \
				-r $framerate \
				-x264-params "keyint=$((2*$framerate)):min-keyint=$((2*$framerate))" \
				-sc_threshold 0 \
				-vf "scale=$width:$height,format=yuv420p" \
				-b:v "${BITRATES[$i]}k" \
				-maxrate "$((2*${BITRATES[$i]}))k" \
				-movflags faststart \
				-bufsize "$((2*${BITRATES[$i]}))k" \
				-map_metadata -1 \
				-y \
				"encoded/$video_name/video_${RESOLUTIONS[$i]}.mp4"
		fi
	done

	if [ ! -d "dash/$video_name" ]; then
		MP4Box \
			-dash 4000 \
			-frag 4000 \
			-rap \
			-mpd-title "${video_name}" \
			-fps $framerate \
			-segment-name '$RepresentationID$/segment_$Init=0$$Number$' \
			-out "dash/$video_name/playlist.mpd" \
			"${dash_sources[@]}"
	fi
}

if [ $# -eq 0 ]; then
	for video in source/*/*.*; do
		encode_video "$video"
	done
else
	for video in $@; do
		encode_video "$video"
	done
fi
