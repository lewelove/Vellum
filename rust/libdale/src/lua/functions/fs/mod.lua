return function(M)
    function M.basename(file)
        if not file or file == "" then return nil end
        local clean = file:gsub("/+$", "")
        if clean == "" then return "" end
        return clean:match("[^/]+$")
    end

    function M.dirname(file)
        if not file or file == "" then return nil end
        local clean = file:gsub("/+$", "")
        if clean == "" then
            if file:sub(1, 1) == "/" then
                return "/"
            end
            return nil
        end
        local dir = clean:match("^(.*)/[^/]*$")
        if not dir or dir == "" then
            if clean:sub(1, 1) == "/" then
                return "/"
            end
            return nil
        end
        return dir
    end

    function M.normalize(path, opts)
        if not path or path == "" then return "" end
        opts = opts or {}
        local p = path
        if p:sub(1, 1) == "~" then
            local home = os.getenv("HOME") or ""
            p = home .. p:sub(2)
        end
        p = p:gsub("//+", "/")
        local is_abs = p:sub(1, 1) == "/"
        local parts = {}
        for part in p:gmatch("[^/]+") do
            if part == ".." then
                if #parts > 0 and parts[#parts] ~= ".." then
                    table.remove(parts)
                elseif not is_abs then
                    table.insert(parts, "..")
                end
            elseif part ~= "." then
                table.insert(parts, part)
            end
        end
        local res = (is_abs and "/" or "") .. table.concat(parts, "/")
        if res == "" then
            return is_abs and "/" or "."
        end
        return res
    end

    function M.joinpath(...)
        local n = select("#", ...)
        local parts = {}
        for i = 1, n do
            local item = select(i, ...)
            if item and item ~= "" then
                table.insert(parts, item)
            end
        end
        if #parts == 0 then return "" end
        return M.normalize(table.concat(parts, "/"))
    end

    function M.parents(start)
        local cur = M.normalize(start)
        return function()
            if not cur or cur == "" or cur == "/" or cur == "." then
                return nil
            end
            local parent = M.dirname(cur)
            if not parent or parent == cur then
                cur = nil
                return nil
            end
            cur = parent
            return parent
        end
    end

    function M.dir(path, opts)
        opts = opts or {}
        local norm = M.normalize(path)
        local depth = opts.depth or 1
        local skip = opts.skip
        local follow = opts.follow or false

        if depth == 1 then
            return M._scandir_native(norm, follow)
        end

        local stack = { { path = norm, rel = "", depth = 1 } }
        local cur_iter = nil
        local cur_node = nil

        return function()
            while true do
                if not cur_iter then
                    if #stack == 0 then
                        return nil
                    end
                    cur_node = table.remove(stack)
                    cur_iter = M._scandir_native(cur_node.path, follow)
                end

                local name, type_ = cur_iter()
                if name then
                    local rel = (cur_node.rel == "") and name or (cur_node.rel .. "/" .. name)
                    if type_ == "directory" and cur_node.depth < depth then
                        local should_skip = skip and skip(rel)
                        if not should_skip then
                            table.insert(stack, {
                                path = M.joinpath(cur_node.path, name),
                                rel = rel,
                                depth = cur_node.depth + 1,
                            })
                        end
                    end
                    return rel, type_
                else
                    cur_iter = nil
                end
            end
        end
    end

    function M.find(names, opts)
        opts = opts or {}
        local start = M.normalize(opts.path or ".")
        local upward = opts.upward or false
        local stop = opts.stop and M.normalize(opts.stop) or nil
        local limit = opts.limit or (upward and 1 or math.huge)
        local match_type = opts.type

        local matches = {}
        local match_fn
        if type(names) == "function" then
            match_fn = names
        elseif type(names) == "table" then
            local set = {}
            for _, n in ipairs(names) do
                set[n] = true
            end
            match_fn = function(name)
                return set[name] == true
            end
        elseif type(names) == "string" then
            match_fn = function(name)
                return name == names
            end
        else
            return matches
        end

        if upward then
            local cur = start
            while cur and #matches < limit do
                for name, type_ in M.dir(cur, { depth = 1 }) do
                    local type_ok = (match_type == nil) or (match_type == type_)
                    local full = M.joinpath(cur, name)
                    if type_ok and match_fn(name, full) then
                        table.insert(matches, full)
                        if #matches >= limit then
                            break
                        end
                    end
                end
                if stop and cur == stop then
                    break
                end
                local parent = M.dirname(cur)
                if not parent or parent == cur then
                    break
                end
                cur = parent
            end
        else
            for rel, type_ in M.dir(start, { depth = opts.depth or math.huge, skip = opts.skip }) do
                local type_ok = (match_type == nil) or (match_type == type_)
                local bname = M.basename(rel)
                local full = M.joinpath(start, rel)
                if type_ok and match_fn(bname, full) then
                    table.insert(matches, full)
                    if #matches >= limit then
                        break
                    end
                end
            end
        end

        return matches
    end

    function M.root(source, marker)
        local start = type(source) == "string" and source or "."
        start = M.normalize(start)
        local res = M.find(marker, {
            path = start,
            upward = true,
            limit = 1,
        })
        if #res > 0 then
            return M.dirname(res[1])
        end
        return nil
    end
end
