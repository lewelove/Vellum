local M = {}

local function islist(t)
    if type(t) ~= "table" then
        return false
    end
    local j = 1
    for _ in pairs(t) do
        if t[j] == nil then
            return false
        end
        j = j + 1
    end
    return true
end

local function can_merge(v)
    return type(v) == "table" and (next(v) == nil or not islist(v))
end

local function tbl_extend(behavior, deep, ...)
    if type(behavior) ~= "string" and type(behavior) ~= "function" then
        error(string.format("invalid 'behavior': %s", tostring(behavior)))
    end

    if type(behavior) == "string" then
        if behavior ~= "error" and behavior ~= "keep" and behavior ~= "force" then
            error(string.format("invalid 'behavior': %s", tostring(behavior)))
        end
    end

    local ret = {}
    local nargs = select("#", ...)
    for i = 1, nargs do
        local t = select(i, ...)
        if t ~= nil then
            if type(t) ~= "table" then
                error(string.format("expected table, got %s", type(t)))
            end
            for k, v in pairs(t) do
                if deep and can_merge(v) and can_merge(ret[k]) then
                    ret[k] = tbl_extend(behavior, true, ret[k], v)
                elseif behavior == "error" then
                    if ret[k] ~= nil then
                        error(string.format("key found in more than one map: %s", tostring(k)))
                    end
                    ret[k] = v
                elseif behavior == "keep" then
                    if ret[k] == nil then
                        ret[k] = v
                    end
                elseif behavior == "force" then
                    ret[k] = v
                elseif type(behavior) == "function" then
                    ret[k] = behavior(k, ret[k], v)
                end
            end
        end
    end
    return ret
end

function M.extend(behavior, ...)
    return tbl_extend(behavior, false, ...)
end

function M.deep_extend(behavior, ...)
    return tbl_extend(behavior, true, ...)
end

function M.contains(t, value, opts)
    if type(t) ~= "table" then
        return false
    end
    local pred
    if opts and opts.predicate then
        if type(value) ~= "function" then
            error("expected function for predicate, got " .. type(value))
        end
        pred = value
    else
        pred = function(v)
            return v == value
        end
    end
    for _, v in pairs(t) do
        if pred(v) then
            return true
        end
    end
    return false
end

function M.filter(fn, t)
    if type(fn) ~= "function" then
        error(string.format("expected function, got %s", type(fn)))
    end
    if type(t) ~= "table" then
        error(string.format("expected table, got %s", type(t)))
    end
    local res = {}
    for _, v in pairs(t) do
        if fn(v) then
            table.insert(res, v)
        end
    end
    return res
end

function M.map(fn, t)
    if type(fn) ~= "function" then
        error(string.format("expected function, got %s", type(fn)))
    end
    if type(t) ~= "table" then
        error(string.format("expected table, got %s", type(t)))
    end
    local res = {}
    for k, v in pairs(t) do
        res[k] = fn(v)
    end
    return res
end

function M.keys(t)
    if type(t) ~= "table" then
        error(string.format("expected table, got %s", type(t)))
    end
    local keys = {}
    for k, _ in pairs(t) do
        table.insert(keys, k)
    end
    return keys
end

function M.values(t)
    if type(t) ~= "table" then
        error(string.format("expected table, got %s", type(t)))
    end
    local values = {}
    for _, v in pairs(t) do
        table.insert(values, v)
    end
    return values
end

function M.count(t)
    if type(t) ~= "table" then
        error(string.format("expected table, got %s", type(t)))
    end
    local count = 0
    for _ in pairs(t) do
        count = count + 1
    end
    return count
end

function M.isempty(t)
    if type(t) ~= "table" then
        return false
    end
    return next(t) == nil
end

function M.isarray(t)
    return islist(t)
end

function M.islist(t)
    return islist(t)
end

function M.flatten(t)
    if type(t) ~= "table" then
        error(string.format("expected table, got %s", type(t)))
    end
    local result = {}
    local function _flatten(_t)
        local n = #_t
        for i = 1, n do
            local v = _t[i]
            if type(v) == "table" then
                _flatten(v)
            elseif v ~= nil then
                table.insert(result, v)
            end
        end
    end
    _flatten(t)
    return result
end

function M.get(o, ...)
    if type(o) ~= "table" then
        return nil
    end
    local nargs = select("#", ...)
    if nargs == 0 then
        return nil
    end
    for i = 1, nargs do
        o = o[select(i, ...)]
        if o == nil then
            return nil
        elseif type(o) ~= "table" and i ~= nargs then
            return nil
        end
    end
    return o
end

function M.add_reverse_lookup(o)
    if type(o) ~= "table" then
        error(string.format("expected table, got %s", type(o)))
    end
    local keys = M.keys(o)
    for _, k in ipairs(keys) do
        local v = o[k]
        if o[v] ~= nil and o[v] ~= k then
            error(string.format("The reverse lookup found an existing value for %q while processing key %q", tostring(v), tostring(k)))
        end
        o[v] = k
    end
    return o
end

local function _deepcopy(orig, cache)
    if type(orig) ~= "table" then
        return orig
    end
    if cache and cache[orig] then
        return cache[orig]
    end
    local copy = {}
    if cache then
        cache[orig] = copy
    end
    for k, v in pairs(orig) do
        copy[_deepcopy(k, cache)] = _deepcopy(v, cache)
    end
    local mt = getmetatable(orig)
    if type(mt) == "table" then
        setmetatable(copy, mt)
    end
    return copy
end

function M.deepcopy(orig, noref)
    return _deepcopy(orig, not noref and {} or nil)
end

function M.deep_equal(a, b)
    if a == b then
        return true
    end
    if type(a) ~= type(b) then
        return false
    end
    if type(a) == "table" then
        for k, v in pairs(a) do
            if not M.deep_equal(v, b[k]) then
                return false
            end
        end
        for k, _ in pairs(b) do
            if a[k] == nil then
                return false
            end
        end
        return true
    end
    return false
end

function M.list_extend(dst, src, start, finish)
    if type(dst) ~= "table" then
        error(string.format("expected table, got %s", type(dst)))
    end
    if type(src) ~= "table" then
        error(string.format("expected table, got %s", type(src)))
    end
    local s = start or 1
    local f = finish or #src
    for i = s, f do
        table.insert(dst, src[i])
    end
    return dst
end

function M.list_slice(list, start, finish)
    if type(list) ~= "table" then
        error(string.format("expected table, got %s", type(list)))
    end
    local new_list = {}
    local s = start or 1
    local f = finish or #list
    for i = s, f do
        new_list[#new_list + 1] = list[i]
    end
    return new_list
end

function M.list_contains(list, value)
    if type(list) ~= "table" then
        return false
    end
    for _, v in ipairs(list) do
        if v == value then
            return true
        end
    end
    return false
end

return M
