_G.dl = _G.dl or {}
_G.dl.fs = _G.dl.fs or {}

local function expand_path(filepath)
    if type(filepath) ~= "string" or filepath == "" then
        return nil
    end
    if filepath == "~" or filepath:sub(1, 2) == "~/" then
        local home = os.getenv("HOME")
        if home then
            return home .. filepath:sub(2)
        end
    end
    return filepath
end

function _G.dl.fs.exists(filepath)
    local expanded = expand_path(filepath)
    if not expanded then
        return false
    end
    REGISTRY.dependencies[expanded] = true
    local file = io.open(expanded, "r")
    if file then
        file:close()
        return true
    end
    return false
end

function _G.dl.fs.read(filepath)
    local expanded = expand_path(filepath)
    if not expanded then
        return nil
    end
    REGISTRY.dependencies[expanded] = true
    local file = io.open(expanded, "r")
    if not file then
        return nil
    end
    local content = file:read("*a")
    file:close()
    return content
end

function _G.dl.fs.read_json(filepath)
    local content = _G.dl.fs.read(filepath)
    if not content or content:match("^%s*$") then
        return nil
    end
    return _G.dl.json.decode(content)
end

function _G.dl.fs.read_toml(filepath)
    local content = _G.dl.fs.read(filepath)
    if not content or content:match("^%s*$") then
        return nil
    end
    return _G.dl.toml.decode(content)
end

function _G.dl.fs.read_lines(filepath)
    local expanded = expand_path(filepath)
    if not expanded then
        return {}
    end
    REGISTRY.dependencies[expanded] = true
    local file = io.open(expanded, "r")
    if not file then
        return {}
    end
    local lines = {}
    local idx = 1
    for line in file:lines() do
        local trimmed = line:match("^%s*(.-)%s*$")
        if trimmed ~= "" and not trimmed:find("^#") then
            lines[idx] = trimmed
            lines[trimmed] = idx
            idx = idx + 1
        end
    end
    file:close()
    return lines
end
