package.path = "rime/lua/?.lua;" .. package.path
local search = require("hanzi_search_find")
assert(search.parse("⿰号虎"))
assert(search.parse("⿲木木木[G]", true))
assert(search.parse("⿾木"))
assert(search.parse("{木旁}"))
assert(search.parse("㇯⿱木林⿾木"))
assert(search.parse("⿿𠂉"))
for _, bad in ipairs({"", "木木", "⿰木", "{木", "{}", "木[G]", "\255"}) do
    assert(not search.parse(bad), bad)
end

local path = os.tmpname()
local file = assert(io.open(path, "w"))
file:write([[malformed
U+6728 木 木
U+6797 林 ⿰木木
U+68EE 森 ⿱木林
U+660E 明 ⿰日月[G] ⿰目月[T]
U+7532 甲 乙
U+4E59 乙 甲
U+20000 𠀀 ⿰林{木旁}
U+4E19 丙 丁 木
U+4E01 丁 丙
U+4E8C 二 ⿰甲丙
U+4E09 三 ⿰丁丙
U+4EBA 人 ⿰亻木[G] ⿰日月[G]
]])
file:close()
local db = assert(search.load(path))
os.remove(path)
local function eq(needles, expected)
    assert(table.concat(assert(search.find(db, needles))) == expected)
end
eq({"木"}, "丁三丙二人木林森𠀀")
eq({"木", "木"}, "丁三丙二人木林森𠀀")
eq({"⿰木木"}, "林森𠀀")
eq({"日", "目"}, "")
eq({"目", "月"}, "明")
eq({"{木旁}"}, "𠀀")
eq({"木", "月"}, "")
eq({"木", "{木旁}", "⿰木木"}, "𠀀")
eq({"甲", "木"}, "二")
eq({"日", "月"}, "人明")
assert(search.find(db, {"不存在"}) == nil)
assert(search.load("/nonexistent/hanzi-search/chai.txt") == nil)
local touched_tail = false
local lazy_db = {chars = {"木", "林"}, by_char = { ["木"] = "木" },
                 children = {["⿰木木"] = "木\t木"}}
setmetatable(lazy_db.by_char, {__index = function()
    touched_tail = true
    return "⿰木木"
end})
local iter = assert(search.iter(lazy_db, {"木"}))
assert(not touched_tail)
assert(iter() == "木" and not touched_tail)
assert(iter() == "林" and touched_tail)
assert(iter() == nil and iter() == nil)
print("find unit tests passed")

if arg[1] then
    local real = assert(search.load("chai.txt"))
    local needles = {}
    for i = 1, #arg do needles[i] = arg[i] end
    for _, char in ipairs(assert(search.find(real, needles))) do print(char) end
end
