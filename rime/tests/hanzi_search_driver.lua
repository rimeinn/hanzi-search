-- Mira only sends keys. F8 exercises Context:select, also used by mouse selection.
local editor = require("hanzi_search").editor
local M = {init = editor.init, fini = editor.fini}
function M.func(key, env)
    local ctx = env.engine.context
    if key:repr() == "F8" then
        local seg = ctx.composition:back()
        ctx:select(seg.selected_index)
        return 1
    elseif key:repr() == "F7" then
        local expected = 0
        for _ in ctx:get_property("hanzi_search_components"):gmatch("\n") do
            expected = expected + 1
        end
        local segments = ctx.composition:toSegmentation():get_segments()
        local confirmed, pending = 0, 0
        for _, seg in ipairs(segments) do
            if seg:has_tag("hanzi_component") and seg.status == "kConfirmed" then
                confirmed = confirmed + 1
            elseif not seg:has_tag("phony") then
                pending = pending + 1
            end
        end
        local ok = confirmed == expected and pending == 1 and ctx:get_option("_auto_commit")
        if not ok then env.engine:commit_text("FAILED: native segment state") end
        return 1
    end
    return editor.func(key, env)
end
return M
