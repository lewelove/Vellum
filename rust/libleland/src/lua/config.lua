_G.ll = _G.ll or {}

_G.ll.config = function(t)
    REGISTRY.config = t
end

_G.ll.interface = function(name, t)
    if type(t) == "boolean" then
        if t == true then
            REGISTRY.interfaces[name] = { enable = true, config = {}, assets = {} }
        end
    elseif type(t) == "table" then
        t.enable = true
        if t.config == nil then t.config = {} end
        if t.assets == nil then t.assets = {} end
        REGISTRY.interfaces[name] = t
    end
end

_G.ll.action = function(name, t)
    if type(t) == "boolean" then
        if t == true then
            REGISTRY.actions[name] = { config = {} }
        end
    elseif type(t) == "table" then
        if t.config == nil then t.config = {} end
        REGISTRY.actions[name] = t
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
    if REGISTRY.filters[name] == nil then
        table.insert(REGISTRY.filters_order, name)
    end
    REGISTRY.filters[name] = t
end

_G.ll.grouper = function(name, t)
    if REGISTRY.groupers[name] == nil then
        table.insert(REGISTRY.groupers_order, name)
    end
    REGISTRY.groupers[name] = t
end

_G.ll.order = function(name, t)
    if REGISTRY.orders[name] == nil then
        table.insert(REGISTRY.orders_order, name)
    end
    REGISTRY.orders[name] = t
end

_G.ll.library = function(name, t)
    if REGISTRY.libraries[name] == nil then
        table.insert(REGISTRY.libraries_order, name)
    end
    REGISTRY.libraries[name] = t
end

_G.ll.shelf = function(name, t)
    if REGISTRY.shelves[name] == nil then
        table.insert(REGISTRY.shelves_order, name)
    end
    REGISTRY.shelves[name] = t
end
