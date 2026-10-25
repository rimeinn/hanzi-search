package.path = "rime/lua/?.lua;" .. package.path
local loads, fail = 0, false
package.loaded.hanzi_search_find = {
    load = function(path)
        loads = loads + 1
        if fail then return nil, "missing fixture" end
        return {path = path}
    end,
    iter = function()
        local done = false
        return function()
            if not done then done = true; return "饕" end
        end
    end,
}
local user_dir = "/first"
rime_api = {get_user_data_dir = function() return user_dir end}
log = {error = function() end}
Candidate = function(_, _, _, text) return {text = text} end
yield = function(cand) assert(cand.text == "饕") end
local translator = require("hanzi_search").translator
local segment = {start = 0, _end = 6, has_tag = function() return true end}
local function environment()
    return {prefix = "oi", data_file = "lua/chai.txt", engine = {context = {
        input = "oihkh'",
        get_property = function() return "號\thkh\n" end,
    }}}
end
local first, second = environment(), environment()
translator.func("", segment, first)
translator.func("", segment, second)
assert(loads == 1 and first.db == second.db)
user_dir = "/second"
local third = environment()
translator.func("", segment, third)
assert(loads == 2 and third.db ~= first.db)
user_dir = "/missing"
fail = true
local fourth = environment()
translator.func("", segment, fourth)
assert(loads == 3 and fourth.db == nil)
fail = false
translator.func("", segment, fourth)
assert(loads == 4 and fourth.db ~= nil)
local alternate = environment()
alternate.data_file = "lua/alternate.txt"
translator.func("", segment, alternate)
assert(loads == 5 and alternate.db ~= fourth.db)
assert(alternate.db.path == "/missing/lua/alternate.txt")
print("shared database cache tests passed")
