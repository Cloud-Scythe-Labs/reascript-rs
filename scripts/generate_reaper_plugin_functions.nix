{ writeShellScriptBin, writeTextFile, coreutils, xdotool }:
let
  echo = "${coreutils}/bin/echo";
  xdotool' = "${xdotool}/bin/xdotool";
  lua_script_name = "generate_reaper_plugin_functions.lua";

  genReaperPluginFunctionsBin = writeShellScriptBin "generate_reaper_plugin_functions.sh" ''
    DEVELOPER_WRITE_WINDOW_NAME="\[developer\] Write C\+\+ API functions header"
    GENERATE_REAPER_PLUGIN_FUNCTIONS_LUA_FILENAME="${lua_script_name}"

    function handle_save_dialog_box() {
        # Find the window ID of the Reaper save dialog box
        reaper_window_id=$(${xdotool'} search --name "$DEVELOPER_WRITE_WINDOW_NAME")
        if [[ -n "$reaper_window_id" ]]; then
            ${echo} "found save dialog window with ID: $reaper_window_id"
        else
            ${echo} "did not find the save dialog window"
            exit 1
        fi

        # Focus the Reaper save dialog box
        ${xdotool'} windowfocus "$reaper_window_id"

        # Use xdotool to simulate navigating to the path entry input, typing the file path and saving the file.
        ${xdotool'} key Tab Tab Tab
        ${xdotool'} type "$1"
        ${xdotool'} key Return Return

        # Accept overwrite of existing header file
        sleep 1
        ${xdotool'} key Return
    }
    function add_generate_reaper_plugin_functions_lua() {
        reaper_window_id=$(${xdotool'} search --name "EVALUATION LICENSE")
        ${echo} "Lua script path: $LUA_PLUGIN_PATH/$GENERATE_REAPER_PLUGIN_FUNCTIONS_LUA_FILENAME"

        if [[ -n "$reaper_window_id" ]]; then
            ${echo} "found reaper window with ID: $reaper_window_id"
        else
            ${echo} "did not find the reaper window"
            exit 1
        fi

        ${xdotool'} windowfocus "$reaper_window_id"

        ${xdotool'} key shift+slash
        sleep 2
        ${xdotool'} key Tab Tab Tab Tab Tab Tab Tab Tab
        ${xdotool'} key Enter
        ${xdotool'} key Up
        ${xdotool'} key Enter
        ${xdotool'} key Tab Tab Tab
        ${xdotool'} type "$1"
        ${xdotool'} key Enter
        ${xdotool'} type "$GENERATE_REAPER_PLUGIN_FUNCTIONS_LUA_FILENAME"
        ${xdotool'} key Enter
    }

    if [ -z "$1" ]; then
        ${echo} "Expected one argument which should be a path to a directory to a Reaper binary."
        exit 1
    fi
    if [ -z "$2" ]; then
        ${echo} "Expected one argument which should be a path to the lua plugin for generating the C++ reaper plugin functions header."
        exit 1
    fi
    if [ -z "$3" ]; then
        ${echo} "Expected one argument which should be a path to a directory to save the generated header file."
        exit 1
    fi

    REAPER=$1
    LUA_PLUGIN_PATH=$2
    SAVE_DIRECTORY=$3
    if [ -e "$SAVE_DIRECTORY" ]; then
        # add the plugin, changing the reaper .ini file
        "$REAPER" &
        export reaper_pid=$!
        sleep 7
        add_generate_reaper_plugin_functions_lua $LUA_PLUGIN_PATH
        kill $reaper_pid

        # run the plugin
        "$REAPER" "$LUA_PLUGIN_PATH/$GENERATE_REAPER_PLUGIN_FUNCTIONS_LUA_FILENAME" &
        export reaper_pid=$!
        sleep 5
        handle_save_dialog_box $SAVE_DIRECTORY
        kill $reaper_pid
    else
        ${echo} "'$SAVE_DIRECTORY' does not exist, please provide a valid path to a directory."
        exit 1
    fi
  '';
  genReaperPluginFunctionsLua = writeTextFile {
    name = "${lua_script_name}";
    text = builtins.readFile ../generate-reaper-plugin-functions/${lua_script_name};
    destination = "/${lua_script_name}";
  };
in
{
  inherit genReaperPluginFunctionsBin genReaperPluginFunctionsLua;
}
