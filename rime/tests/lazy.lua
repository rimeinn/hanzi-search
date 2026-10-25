package.path = "rime/lua/?.lua;" .. package.path
local steps, loads = 0, 0
package.loaded.hanzi_search_find = {
    load = function() loads = loads + 1; return {} end,
    iter = function()
        local index = 0
        return function()
            steps = steps + 1
            index = index + 1
            return ({"木", "林", "森"})[index]
        end
    end,
}
rime_api = {get_user_data_dir = function() return "/fixture" end}
Candidate = function(_, _, _, text) return {text = text} end
yield = coroutine.yield
local translator = require("hanzi_search").translator
local env = {engine = {schema = {config = {get_string = function() end}}, context = {
    input = "oimu'",
    get_property = function() return "木\tmu\n" end,
}}}
translator.init(env)
assert(loads == 1 and steps == 0)
local segment = {start = 4, _end = 5, has_tag = function() return true end}
local function translation()
    return coroutine.create(function() translator.func("'", segment, env) end)
end
local function next_text(thread)
    local ok, cand = coroutine.resume(thread)
    assert(ok, cand)
    return cand and cand.text
end
local first = translation()
assert(steps == 0)
assert(next_text(first) == "木" and steps == 1)
assert(loads == 1)
local refreshed = translation()
assert(next_text(refreshed) == "木" and steps == 1)
assert(next_text(refreshed) == "林" and steps == 2)
assert(next_text(first) == "林" and steps == 2)
assert(next_text(first) == "森" and steps == 3)
assert(next_text(first) == nil and steps == 4)
print("lazy translator and cached replay tests passed")
