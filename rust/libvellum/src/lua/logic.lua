_G.vl = _G.vl or {}

function __VELLUM_GET_LOGIC_MANIFEST()
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
            strict = v.strict or false,
            index = v.index,
            count = v.count
        }
    end
    for k, v in pairs(REGISTRY.orders or {}) do
        manifest.orders[k] = {
            label = v.label or k,
            reverse = v.reverse or false,
            strict = v.strict or false
        }
    end
    for k, v in pairs(REGISTRY.libraries or {}) do
        manifest.libraries[k] = {
            label = v.label or k,
            strict = v.strict or false,
            filters = v.filters or {},
            groupers = v.groupers or {},
            orders = v.orders or {}
        }
    end
    for k, v in pairs(REGISTRY.shelves or {}) do
        manifest.shelves[k] = {
            label = v.label or k,
            reverse = v.reverse or false,
            file = v.file
        }
    end
    return manifest
end

function __VELLUM_EVALUATE_ALBUM_LOGIC(raw_album)
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
            end
        end
    end

    for k, v in pairs(REGISTRY.groupers or {}) do
        if type(v.select) == "function" then
            local ok, sel_res = pcall(v.select, album)
            if ok and sel_res ~= nil then
                res.groupers[k] = sel_res
            end
        end
    end

    for k, v in pairs(REGISTRY.orders or {}) do
        if type(v.sort) == "function" then
            local ok, sort_res = pcall(v.sort, album)
            if ok and sort_res ~= nil then
                res.orders[k] = sort_res
            end
        end
    end

    for k, v in pairs(REGISTRY.libraries or {}) do
        if type(v.match) == "function" then
            local ok, match_res = pcall(v.match, album)
            if ok and match_res then
                res.libraries[k] = true
            end
        end
    end

    for k, v in pairs(REGISTRY.shelves or {}) do
        if type(v.match) == "function" then
            local ok, match_res = pcall(v.match, album)
            if ok and match_res then
                res.shelves[k] = true
            end
        end
        if type(v.sort) == "function" then
            local ok, sort_res = pcall(v.sort, album)
            if ok and sort_res ~= nil then
                res.shelf_sorts[k] = sort_res
            end
        end
    end

    return res
end
