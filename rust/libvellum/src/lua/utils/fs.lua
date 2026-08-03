_G.vl = _G.vl or {}
_G.vl.fs = _G.vl.fs or {}

function _G.vl.fs.exists(filepath)
    if not filepath or filepath == "" then return false end
    local expanded = filepath
    if filepath:sub(1, 1) == "~" then
        local home = os.getenv("HOME")
        if home then
            expanded = home .. filepath:sub(2)
        end
    end
    REGISTRY.dependencies[expanded] = true
    local file = io.open(expanded, "r")
    if file then
        file:close()
        return true
    end
    return false
end

function _G.vl.fs.read(filepath)
    if not filepath or filepath == "" then return nil end
    local expanded = filepath
    if filepath:sub(1, 1) == "~" then
        local home = os.getenv("HOME")
        if home then
            expanded = home .. filepath:sub(2)
        end
    end
    REGISTRY.dependencies[expanded] = true
    local file = io.open(expanded, "r")
    if not file then return nil end
    local content = file:read("*a")
    file:close()
    return content
end

function _G.vl.fs.read_json(filepath)
    local content = _G.vl.fs.read(filepath)
    if not content then return nil end
    return _G.vl.json.decode(content)
end

function _G.vl.fs.read_toml(filepath)
    local content = _G.vl.fs.read(filepath)
    if not content then return nil end
    return _G.vl.toml.decode(content)
end

function _G.vl.fs.read_lines(filepath)
    if not filepath or filepath == "" then return {} end
    local expanded = filepath
    if filepath:sub(1, 1) == "~" then
        local home = os.getenv("HOME")
        if home then
            expanded = home .. filepath:sub(2)
        end
    end
    REGISTRY.dependencies[expanded] = true
    local file = io.open(expanded, "r")
    if not file then return {} end
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
