_G.dale = _G.dale or {}
_G.d = _G.dale

local function get(tbl, ...)
    if type(tbl) ~= "table" then
        return nil
    end

    local curr = tbl
    local n = select("#", ...)

    for idx = 1, n do
        if type(curr) ~= "table" then
            return nil
        end

        local arg = select(idx, ...)
        if type(arg) == "string" and arg:find(".", 1, true) then
            for part in arg:gmatch("[^%.]+") do
                if type(curr) ~= "table" then
                    return nil
                end
                local num = tonumber(part)
                if num and math.floor(num) == num then
                    curr = curr[num] ~= nil and curr[num] or curr[part]
                else
                    curr = curr[part]
                end
            end
        elseif arg ~= nil then
            curr = curr[arg]
        else
            return nil
        end
    end

    return curr
end

_G.dale.get = get
_G.d.get = get
