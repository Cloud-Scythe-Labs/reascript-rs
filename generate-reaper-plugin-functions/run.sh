#!/bin/bash

DEVELOPER_WRITE_WINDOW_NAME="\[developer\] Write C\+\+ API functions header"

function handle_save_dialog_box() {
    # Find the window ID of the Reaper save dialog box
    reaper_window_id=$(xdotool search --name "$DEVELOPER_WRITE_WINDOW_NAME")
    echo "found save dialog window with ID: $reaper_window_id"

    # Focus the Reaper save dialog box
    xdotool windowfocus "$reaper_window_id"

    # Use xdotool to simulate navigating to the path entry input, typing the file path and saving the file.
    xdotool key Tab Tab Tab
    xdotool type "$1"
    xdotool key Return Return
}

# TODO: Check first arg is a path that exists
# ~/Code/CloudScytheLabs/reaper-header-generator
SAVE_DIRECTORY=$1
reaper ./generate_reaper_plugin_functions.lua &
sleep 5
handle_save_dialog_box $SAVE_DIRECTORY
# TODO: Exit Reaper once complete
