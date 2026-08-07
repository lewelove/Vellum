_G.ll = _G.ll or {}
_G.ll.compile = _G.ll.compile or {}
_G.ll.compile.album = {
    key = function(t)
        local count = 0
        for _ in pairs(t) do count = count + 1 end
        if count > 1 then error("ll.compile.album.key({}) accepts only 1 key per call.") end
        for k, v in pairs(t) do
            if type(v) == "function" then
                REGISTRY.keys.album[k] = v
            end
        end
    end
}

_G.ll.compile.tracks = {
    key = function(t)
        local count = 0
        for _ in pairs(t) do count = count + 1 end
        if count > 1 then error("ll.compile.tracks.key({}) accepts only 1 key per call.") end
        for k, v in pairs(t) do
            if type(v) == "function" then
                REGISTRY.keys.tracks[k] = v
            end
        end
    end,
    lyrics = function(t)
        if type(t) == "function" then
            REGISTRY.lyrics = { text = t, type = "txt" }
        elseif type(t) == "table" then
            REGISTRY.lyrics = {
                text = t.text,
                type = t.type or "txt"
            }
        end
    end
}

_G.ll.compile.a = _G.ll.compile.album
_G.ll.compile.track = _G.ll.compile.tracks
_G.ll.compile.t = _G.ll.compile.tracks

function __LELAND_DISPATCHER(ctx, manifests)
    local results = { album = {}, tracks = {} }

    for key_name, func in pairs(REGISTRY.keys.album) do
        local status, res = pcall(func, ctx, manifests)
        if not status then
            error(string.format("Error evaluating album key '%s': %s", key_name, res))
        end
        if res ~= nil and res ~= "" then
            results.album[key_name] = res
        end
    end
    
    for i = 1, ctx.total_tracks do
        results.tracks[i] = {}
        for key_name, func in pairs(REGISTRY.keys.tracks) do
            local status, res = pcall(func, ctx, manifests, i)
            if not status then
                error(string.format("Error evaluating track key '%s' at index %d: %s", key_name, i, res))
            end
            if res ~= nil and res ~= "" then
                results.tracks[i][key_name] = res
            end
        end

        if REGISTRY.lyrics and type(REGISTRY.lyrics.text) == "function" then
            local status, lyr_text = pcall(REGISTRY.lyrics.text, ctx, manifests, i)
            if not status then
                error(string.format("Error evaluating track lyrics at index %d: %s", i, lyr_text))
            end
            if lyr_text ~= nil and lyr_text ~= "" then
                local lyr_type = "txt"
                if type(REGISTRY.lyrics.type) == "function" then
                    local t_status, t_res = pcall(REGISTRY.lyrics.type, ctx, manifests, i)
                    if t_status and t_res and t_res ~= "" then
                        lyr_type = t_res
                    end
                elseif type(REGISTRY.lyrics.type) == "string" and REGISTRY.lyrics.type ~= "" then
                    lyr_type = REGISTRY.lyrics.type
                end
                results.tracks[i]["lyrics"] = {
                    ["type"] = lyr_type,
                    ["text"] = lyr_text
                }
            end
        end
    end
    
    return results
end
