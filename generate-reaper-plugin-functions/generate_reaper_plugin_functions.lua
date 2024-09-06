-- Basic Reaper script for generating the reaper_plugin_functions.h C++ header file

-- Command ID for "[developer] Write C++ API functions header"
local writeCppFunctionsHeader = 41064

-- Run the command
reaper.Main_OnCommand(writeCppFunctionsHeader, 0)
