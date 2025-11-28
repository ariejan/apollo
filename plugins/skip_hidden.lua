-- skip_hidden.lua
-- An example Apollo plugin that skips hidden files during import.
--
-- This demonstrates how to return "skip" to prevent a track from being imported.

local plugin = {
    name = "skip_hidden",
    version = "1.0.0",
    description = "Skips hidden files and system files during import",
    author = "Apollo Team",
}

-- Patterns for files to skip
local skip_patterns = {
    "^%.",           -- Hidden files (start with .)
    "^desktop%.ini$", -- Windows desktop.ini
    "^%.DS_Store$",   -- macOS .DS_Store
    "^Thumbs%.db$",   -- Windows thumbnails
}

-- Check if filename matches any skip pattern
local function should_skip(filename)
    for _, pattern in ipairs(skip_patterns) do
        if filename:match(pattern) then
            return true
        end
    end
    return false
end

-- Called before a track is imported
function plugin.on_import(track)
    -- Extract filename from path
    local filename = track.path:match("([^/\\]+)$")

    if filename and should_skip(filename) then
        apollo.info("skip_hidden: Skipping hidden/system file: " .. filename)
        return "skip"
    end

    return "continue"
end

return plugin
