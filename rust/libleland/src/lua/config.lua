_G.ll = _G.ll or {}

_G.ll.config = function(t)
    if t.interfaces then
        for k, v in pairs(t.interfaces) do
            if type(v) == "boolean" then
                if v == true then
                    REGISTRY.interfaces[k] = { enable = true, config = {}, assets = {} }
                end
            elseif type(v) == "table" then
                v.enable = true
                if v.config == nil then v.config = {} end
                if v.assets == nil then v.assets = {} end
                REGISTRY.interfaces[k] = v
            end
        end
        t.interfaces = nil
    end
    REGISTRY.config = t
end

_G.ll.interfaces = function(t)
    for k, v in pairs(t) do
        if type(v) == "boolean" then
            if v == true then
                REGISTRY.interfaces[k] = { enable = true, config = {}, assets = {} }
            end
        elseif type(v) == "table" then
            v.enable = true
            if v.config == nil then v.config = {} end
            if v.assets == nil then v.assets = {} end
            REGISTRY.interfaces[k] = v
        end
    end
end

_G.ll.actions = function(t)
    for k, v in pairs(t) do
        if type(v) == "boolean" then
            if v == true then
                REGISTRY.actions[k] = { config = {} }
            end
        elseif type(v) == "table" then
            if v.config == nil then v.config = {} end
            REGISTRY.actions[k] = v
        end
    end
end

_G.ll.cache = _G.ll.cache or {}

local cover_fn = function(t)
    table.insert(REGISTRY.covers.targets, t)
end

_G.ll.cache.cover = setmetatable({
    master = function(t)
        if t.size then REGISTRY.covers.master.size = t.size end
        if t.filter then REGISTRY.covers.master.filter = t.filter end
    end
}, {
    __call = function(_, t)
        cover_fn(t)
    end
})

_G.ll.filter = function(name, t)
    REGISTRY.filters[name] = t
end

_G.ll.grouper = function(name, t)
    REGISTRY.groupers[name] = t
end

_G.ll.order = function(name, t)
    REGISTRY.orders[name] = t
end

_G.ll.library = function(name, t)
    REGISTRY.libraries[name] = t
end

_G.ll.shelf = function(name, t)
    REGISTRY.shelves[name] = t
end
