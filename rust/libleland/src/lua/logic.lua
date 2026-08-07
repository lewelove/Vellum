_G.ll = _G.ll or {}

function __LELAND_GET_LOGIC_MANIFEST()
    local manifest = {
        filters = {},
        groupers = {},
        orders = {},
        libraries = {},
        shelves = {}
    }
    for k, v in pairs(REGISTRY.filters or {}) do
        manifest.filters[k] = { label = v.label or k }
    end
    for k, v in pairs(REGISTRY.groupers or {}) do
        manifest.groupers[k] = {
            label = v.label or k,
            index = v.index,
            count = v.count,
            reverse = v.reverse or false
        }
    end
    for k, v in pairs(REGISTRY.orders or {}) do
        manifest.orders[k] = {
            label = v.label or k,
            reverse = v.reverse or false
        }
    end
    for k, v in pairs(REGISTRY.libraries or {}) do
        manifest.libraries[k] = {
            label = v.label or k,
            filters = v.filters or {},
            groupers = v.groupers or {},
            orders = v.orders or {}
        }
    end
    for k, v in pairs(REGISTRY.shelves or {}) do
        manifest.shelves[k] = {
            label = v.label or k,
            reverse = v.reverse or false
        }
    end
    return manifest
end

function __LELAND_EVALUATE_ALBUM_LOGIC(raw_album)
    local album = raw_album
    if raw_album and type(raw_album.album) == "table" then
        album = raw_album.album
        if raw_album.tracks and album.tracks == nil then
            album.tracks = raw_album.tracks
        end
    end

    local res = {
        filters = {},
        groupers = {},
        orders = {},
        libraries = {},
        shelves = {},
        shelf_sorts = {}
    }

    for k, v in pairs(REGISTRY.filters or {}) do
        if type(v.match) == "function" then
            local ok, match_res = pcall(v.match, album)
            if ok and match_res then
                res.filters[k] = true
            elseif not ok then
                print(string.format("Error evaluating filter '%s': %s", k, tostring(match_res)))
            end
        end
    end

    for k, v in pairs(REGISTRY.groupers or {}) do
        if type(v.select) == "function" then
            local ok, sel_res = pcall(v.select, album)
            if ok and sel_res ~= nil then
                local function build_entry(item)
                    local sort_key = item
                    if type(v.sort) == "function" then
                        local s_ok, s_res = pcall(v.sort, item, album)
                        if s_ok and s_res ~= nil then
                            sort_key = s_res
                        elseif not s_ok then
                            print(string.format("Error evaluating grouper sort '%s': %s", k, tostring(s_res)))
                        end
                    end
                    return { value = tostring(item), sort = sort_key }
                end

                if type(sel_res) == "table" then
                    local items = {}
                    for idx, item in ipairs(sel_res) do
                        items[idx] = build_entry(item)
                    end
                    res.groupers[k] = items
                else
                    res.groupers[k] = build_entry(sel_res)
                end
            elseif not ok then
                print(string.format("Error evaluating grouper '%s': %s", k, tostring(sel_res)))
            end
        end
    end

    for k, v in pairs(REGISTRY.orders or {}) do
        if type(v.sort) == "function" then
            local ok, sort_res = pcall(v.sort, album)
            if ok and sort_res ~= nil then
                res.orders[k] = sort_res
            elseif not ok then
                print(string.format("Error evaluating order '%s': %s", k, tostring(sort_res)))
            end
        end
    end

    for k, v in pairs(REGISTRY.libraries or {}) do
        if type(v.match) == "function" then
            local ok, match_res = pcall(v.match, album)
            if ok and match_res then
                res.libraries[k] = true
            elseif not ok then
                print(string.format("Error evaluating library '%s': %s", k, tostring(match_res)))
            end
        end
    end

    for k, v in pairs(REGISTRY.shelves or {}) do
        if type(v.match) == "function" then
            local ok, match_res = pcall(v.match, album)
            if ok and match_res then
                res.shelves[k] = true
            elseif not ok then
                print(string.format("Error evaluating shelf match '%s': %s", k, tostring(match_res)))
            end
        end
        if type(v.sort) == "function" then
            local ok, sort_res = pcall(v.sort, album)
            if ok and sort_res ~= nil then
                res.shelf_sorts[k] = sort_res
            elseif not ok then
                print(string.format("Error evaluating shelf sort '%s': %s", k, tostring(sort_res)))
            end
        end
    end

    return res
end
