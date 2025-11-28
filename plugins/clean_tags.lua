-- clean_tags.lua
-- An example Apollo plugin that cleans up track metadata during import.
--
-- This plugin demonstrates how to:
-- - Define plugin metadata
-- - Implement the on_import hook
-- - Modify track properties
-- - Use the Apollo logging API

local plugin = {
    name = "clean_tags",
    version = "1.0.0",
    description = "Cleans and normalizes track metadata during import",
    author = "Apollo Team",
}

-- Helper function to trim whitespace from strings
local function trim(s)
    if s == nil then return nil end
    return s:match("^%s*(.-)%s*$")
end

-- Helper function to title case a string
local function title_case(s)
    if s == nil then return nil end
    return s:gsub("(%a)([%w_']*)", function(first, rest)
        return first:upper() .. rest:lower()
    end)
end

-- Clean up common artist name issues
local function clean_artist(artist)
    if artist == nil or artist == "" then
        return "Unknown Artist"
    end

    artist = trim(artist)

    -- Remove common prefixes that shouldn't be there
    artist = artist:gsub("^The ", "")
    artist = "The " .. artist  -- Put it back at the start properly

    -- Actually, let's just return the trimmed version
    return trim(artist) or "Unknown Artist"
end

-- Clean up track title
local function clean_title(title, path)
    if title == nil or title == "" then
        -- Extract title from filename if missing
        local filename = path:match("([^/]+)$") or "Unknown"
        -- Remove extension
        filename = filename:gsub("%.[^.]+$", "")
        -- Remove track numbers like "01 - " or "01. "
        filename = filename:gsub("^%d+[%.%-]?%s*", "")
        return trim(filename) or "Unknown Title"
    end

    return trim(title)
end

-- Called before a track is imported
function plugin.on_import(track)
    apollo.debug("clean_tags: Processing track: " .. track.title)

    -- Clean up the title
    local original_title = track.title
    track.title = clean_title(track.title, track.path)
    if track.title ~= original_title then
        apollo.info("clean_tags: Fixed title: '" .. original_title .. "' -> '" .. track.title .. "'")
    end

    -- Clean up the artist
    local original_artist = track.artist
    if track.artist == "" or track.artist == nil then
        track.artist = "Unknown Artist"
        apollo.info("clean_tags: Set missing artist to 'Unknown Artist'")
    else
        track.artist = trim(track.artist)
    end

    -- Ensure genres is a valid table
    if track.genres == nil then
        track.genres = {}
    end

    -- Remove empty genre entries
    local clean_genres = {}
    for _, genre in ipairs(track.genres or {}) do
        local trimmed = trim(genre)
        if trimmed and trimmed ~= "" then
            table.insert(clean_genres, trimmed)
        end
    end
    track.genres = clean_genres

    -- Set year to nil if it's 0 or negative
    if track.year and track.year <= 0 then
        track.year = nil
        apollo.debug("clean_tags: Removed invalid year")
    end

    -- Continue with the import
    return "continue"
end

-- Log when a track is successfully imported
function plugin.post_import(track)
    apollo.debug("clean_tags: Successfully imported: " .. track.artist .. " - " .. track.title)
    return "continue"
end

return plugin
