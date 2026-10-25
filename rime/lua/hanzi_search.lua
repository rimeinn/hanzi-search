local find = require("hanzi_search_find")
local M = {editor = {}, translator = {}, segmentor = {}, filter = {}}
local property = "hanzi_search_components"
-- Immutable data is shared across components and sessions in this Lua VM.
local databases = {}

local function configure(env)
    env.prefix = env.engine.schema.config:get_string("hanzi_search/prefix") or "oi"
    assert(env.prefix:match("^[a-z]+$"), "hanzi_search/prefix must be nonempty lowercase ASCII")
end

M.segmentor.init, M.filter.init = configure, configure

local function database(env)
    local path = rime_api.get_user_data_dir() .. "/" .. env.data_file
    if databases[path] then return databases[path] end
    local db, err = find.load(path)
    if not db then log.error("hanzi_search: " .. err); return nil end
    databases[path] = db
    return db
end

function M.translator.init(env)
    configure(env)
    env.data_file = env.engine.schema.config:get_string("hanzi_search/data_file") or "lua/chai.txt"
    assert(env.data_file ~= "", "hanzi_search/data_file must not be empty")
    env.db = database(env)
end

local function active(ctx, prefix)
    return ctx.input:sub(1, #prefix) == prefix and not ctx:get_option("ascii_mode")
end

local function state(ctx, trigger)
    local components, codes = {}, {}
    for text, code in ctx:get_property(property):gmatch("([^\t\n]+)\t([^\n]+)\n") do
        components[#components + 1], codes[#codes + 1] = text, code
    end
    local prefix = trigger
    for _, code in ipairs(codes) do prefix = prefix .. code .. "'" end
    if ctx.input:sub(1, #prefix) ~= prefix then return {}, {}, ctx.input:sub(#trigger + 1) end
    return components, codes, ctx.input:sub(#prefix + 1)
end

local function save(ctx, components, codes)
    local encoded = {}
    for i, text in ipairs(components) do
        encoded[i] = text .. "\t" .. codes[i] .. "\n"
    end
    ctx:set_property(property, table.concat(encoded))
end

local function kind(cand)
    if not cand then return nil end
    return cand:get_genuine().type
end

function M.editor.init(env)
    configure(env)
    local ctx = env.engine.context
    -- Grouped callbacks precede the engine; the ungrouped callback follows it.
    env.before = ctx.select_notifier:connect(function(context)
        if not active(context, env.prefix) then return end
        local cand = context:get_selected_candidate()
        local typ = kind(cand)
        if typ ~= "hanzi_component" and typ ~= "hanzi_result" then return end
        env.selection = {type = typ, text = cand.text,
                         component = cand:get_genuine().text}
        env.auto_commit = context:get_option("_auto_commit")
        context:set_option("_auto_commit", false)
    end, 0)
    env.after = ctx.select_notifier:connect(function(context)
        local selected = env.selection
        if not selected then return end
        env.selection = nil
        context:set_option("_auto_commit", env.auto_commit)
        if selected.type == "hanzi_result" then
            env.engine:commit_text(selected.text)
            context:clear()
        else
            local components, codes, pending = state(context, env.prefix)
            components[#components + 1] = selected.component
            codes[#codes + 1] = pending
            save(context, components, codes)
            -- The separator gives the new, logically empty segment a real span.
            context:push_input("'")
        end
    end)
    env.update = ctx.update_notifier:connect(function(context)
        if not active(context, env.prefix) and context:get_property(property) ~= "" then
            context:set_property(property, "")
        end
    end)
end

function M.editor.fini(env)
    for _, name in ipairs({"before", "after", "update"}) do
        if env[name] then env[name]:disconnect() end
    end
end

function M.editor.func(key, env)
    local ctx = env.engine.context
    if not active(ctx, env.prefix) or key:release() then return 2 end
    local repr = key:repr()
    if repr == "Escape" then ctx:clear(); return 1 end
    if repr == "BackSpace" then
        local components, codes, pending = state(ctx, env.prefix)
        if pending ~= "" then
            if #pending == 1 and #components > 0 then
                ctx.composition:pop_back()
                local separator = ctx.composition:back()
                separator.tags = Set({"hanzi_search"})
                separator.status = "kVoid"
            end
            ctx:pop_input(1)
        elseif #components > 0 then
            table.remove(codes)
            table.remove(components)
            save(ctx, components, codes)
            ctx.composition:pop_back()
            ctx.composition:back().status = "kGuess"
            ctx:pop_input(1)
        else
            ctx.input = env.prefix:sub(1, -2)
        end
        return 1
    end
    if repr == "space" or repr == "Return" then
        if ctx:get_selected_candidate() then ctx:confirm_current_selection() end
        return 1
    end
    if repr:match("^[a-z]$") then
        if ctx.caret_pos ~= #ctx.input then ctx.caret_pos = #ctx.input end
        -- Equal-length resegmentation merges tags but retains the old result menu.
        if ctx.composition:back():has_tag("hanzi_search") then
            ctx:clear_non_confirmed_composition()
        end
        ctx:push_input(repr)
        return 1
    end
    -- Keep selector paging/numeric selection, but prevent raw-input commits.
    if repr:match("^[0-9]$") or repr == "Up" or repr == "Down"
        or repr == "Page_Up" or repr == "Page_Down" then
        return ctx:has_menu() and 2 or 1
    end
    return 1
end

function M.segmentor.func(segmentation, env)
    if not active(env.engine.context, env.prefix) then return true end
    local start = segmentation:get_current_start_position()
    if start >= #segmentation.input then return false end
    local input = segmentation.input
    local prefix_end = start == 0 and #env.prefix or start + 1
    local is_prefix = start == 0 or input:sub(start + 1, start + 1) == "'"
    local seg
    if is_prefix and (start == 0 or prefix_end < #input) then
        seg = Segment(start, prefix_end)
        seg.tags = Set({"phony"})
        seg.status = "kConfirmed"
    else
        seg = Segment(start, #input)
        seg.tags = Set(is_prefix and {"hanzi_search"} or {"abc", "hanzi_component"})
    end
    segmentation:add_segment(seg)
    return false
end

function M.translator.func(_, seg, env)
    if not seg:has_tag("hanzi_search") then return end
    local components, _, pending = state(env.engine.context, env.prefix)
    if pending == "" and #components > 0 then
        if not env.db then
            env.db = database(env)
            if not env.db then return end
        end
        local cache_key = table.concat(components, "\t")
        if env.cache_key ~= cache_key then
            env.cache = {items = {}, next = assert(find.iter(env.db, components))}
            env.cache_key = cache_key
        end
        local cache, index = env.cache, 1
        while true do
            local text = cache.items[index]
            if not text then
                text = cache.next()
                if not text then break end
                cache.items[index] = text
            end
            local cand = Candidate("hanzi_result", seg.start, seg._end, text, "拆字")
            cand.preedit = " []"
            yield(cand)
            index = index + 1
        end
    end
end

function M.filter.func(input, env)
    local ctx = env.engine.context
    local searching = active(ctx, env.prefix)
    local seg = searching and not ctx.composition:empty() and ctx.composition:back()
    for cand in input:iter() do
        if not searching or kind(cand) == "hanzi_result" then
            yield(cand)
        elseif seg and seg:has_tag("hanzi_component")
            and cand._end == seg._end and find.parse(cand.text)
            and not cand.text:find("[\t\n]") then
            -- Reuse the engine's existing translator stream, without new instances.
            local result = Candidate("hanzi_component", cand.start, cand._end,
                                     cand.text, cand.comment)
            result.preedit, result.quality = cand.preedit, cand.quality
            yield(result)
        end
    end
end

return M
