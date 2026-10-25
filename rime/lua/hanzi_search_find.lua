-- The find strategy from src/lib.rs and src/ids.rs, without wildcard matching.
local M = {}
local arity = {}
for _, cp in utf8.codes("⿰⿱⿲⿳⿴⿵⿶⿷⿸⿹⿺⿻⿼⿽⿾⿿㇯") do
    arity[utf8.char(cp)] = 2
end
arity["⿲"], arity["⿳"], arity["⿾"], arity["⿿"] = 3, 3, 1, 1

function M.parse(text, tagged)
    if not utf8.len(text) then return nil end
    local pos = 1
    local function node()
        local start = pos
        if pos > #text then return nil end
        local char = utf8.char(utf8.codepoint(text, pos))
        pos = pos + #char
        local children
        if arity[char] then
            children = {}
            for i = 1, arity[char] do
                children[i] = node()
                if not children[i] then return nil end
            end
        elseif char == "{" then
            local last = text:find("}", pos, true)
            if not last or last == pos then return nil end
            pos = last + 1
        elseif char == "[" then
            return nil
        end
        return {text = text:sub(start, pos - 1), children = children}
    end
    local result = node()
    local suffix = text:sub(pos)
    if result and (suffix == "" or (tagged and suffix:match("^%[[^%]]+%]$"))) then
        return result
    end
end

function M.load(path)
    local file, err = io.open(path, "r")
    if not file then return nil, err end
    -- String adjacency lists avoid retaining hundreds of thousands of Lua tables.
    local db = {chars = {}, by_char = {}, children = {}}
    local function add_children(tree)
        if not tree.children or db.children[tree.text] then return end
        local children = {}
        for i, child in ipairs(tree.children) do
            children[i] = child.text
            add_children(child)
        end
        db.children[tree.text] = table.concat(children, "\t")
    end
    for line in file:lines() do
        local char, decompositions = line:match("^%S+%s+(%S+)%s+(.+)$")
        if char and utf8.len(char) == 1 then
            for text in decompositions:gmatch("%S+") do
                local tree = M.parse(text, true)
                if tree then
                    if not db.by_char[char] then
                        db.by_char[char] = tree.text
                        db.chars[#db.chars + 1] = char
                    else
                        db.by_char[char] = db.by_char[char] .. "\t" .. tree.text
                    end
                    add_children(tree)
                end
            end
        end
    end
    file:close()
    table.sort(db.chars)
    return db
end

local function contains(db, tree, needle, visiting, memo)
    if tree == needle then return true end
    if memo[tree] ~= nil then return memo[tree] end
    if visiting[tree] then return false, true end
    visiting[tree] = true
    local cyclic = false
    local found = false
    for child in (db.children[tree] or db.by_char[tree] or ""):gmatch("[^\t]+") do
        if child ~= tree then
            local match, cycle = contains(db, child, needle, visiting, memo)
            cyclic = cyclic or cycle
            if match then found = true; break end
        end
    end
    visiting[tree] = nil
    -- A cycle-dependent failure cannot be reused from another traversal root.
    if found or not cyclic then memo[tree] = found end
    return found, cyclic
end

function M.iter(db, components)
    local needles, memo = {}, {}
    for i, component in ipairs(components) do
        if not M.parse(component) then return nil, "Invalid IDS: " .. component end
        needles[i] = component
        memo[i] = {}
    end
    local index = 0
    return function()
        while index < #db.chars do
            index = index + 1
            local char = db.chars[index]
            for tree in db.by_char[char]:gmatch("[^\t]+") do
                local matches = true
                for i, needle in ipairs(needles) do
                    if not contains(db, tree, needle, {}, memo[i]) then
                        matches = false
                        break
                    end
                end
                if matches then return char end
            end
        end
    end
end

function M.find(db, components)
    local iter, err = M.iter(db, components)
    if not iter then return nil, err end
    local results = {}
    for char in iter do results[#results + 1] = char end
    return results
end

return M
