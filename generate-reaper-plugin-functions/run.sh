#!/bin/bash

DEVELOPER_WRITE_WINDOW_NAME="\[developer\] Write C\+\+ API functions header"

function handle_save_dialog_box() {
    # Find the window ID of the Reaper save dialog box
    reaper_window_id=$(xdotool search --name "$DEVELOPER_WRITE_WINDOW_NAME")
    if [[ -n "$reaper_window_id" ]]; then
        echo "found save dialog window with ID: $reaper_window_id"
    else
        echo "did not find the save dialog window"
        exit 1
    fi

    # Focus the Reaper save dialog box
    xdotool windowfocus "$reaper_window_id"

    # Use xdotool to simulate navigating to the path entry input, typing the file path and saving the file.
    xdotool key Tab Tab Tab
    xdotool type "$1"
    xdotool key Return Return

    # Accept overwrite of existing header file
    sleep 1
    xdotool key Return
}

if [ -z "$1" ]; then
    echo "Expected one argument which should be a path to a directory to save the generated header file."
    exit 1
fi

REAPER=$1
SAVE_DIRECTORY=$2
if [ -e "$SAVE_DIRECTORY" ]; then
    "$REAPER" ./generate_reaper_plugin_functions.lua &
    sleep 5
    handle_save_dialog_box $SAVE_DIRECTORY
else
    echo "'$SAVE_DIRECTORY' does not exist, please provide a valid path to a directory."
    exit 1
fi
# TODO: Exit Reaper when finished
