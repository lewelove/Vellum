_G.vl = _G.vl or {}
_G.vl.fn = {
    require = function(v)
        if v == nil or v == "" then error("Value is required") end
        return v
    end,
    type_check = function(v, t)
        if v == nil or v == "" then return v end
        local lua_type = type(v)
        if t == "string" or t == "datetime" or t == "path" or t == "url" then
            if lua_type ~= "string" then error("Expected " .. t .. " but got " .. lua_type) end
        elseif t == "integer" or t == "float" or t == "number" then
            if lua_type ~= "number" then error("Expected " .. t .. " but got " .. lua_type) end
        elseif t == "boolean" then
            if lua_type ~= "boolean" then error("Expected boolean but got " .. lua_type) end
        elseif t == "array" or t == "object" or t == "list" then
            if lua_type ~= "table" then error("Expected table but got " .. lua_type) end
        end
        return v
    end
}

_G.vl.fs = _G.vl.fs or {}

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
