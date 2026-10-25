"""Compare the Lua find port against the Rust CLI on the real database."""
import subprocess

subprocess.run(["cargo", "build", "--quiet"], check=True)
for needles in [[], ["木"], ["木", "木"], ["號", "食"], ["⿰号虎", "食"],
                ["日", "月"], ["⿾木"], ["𠀀"], ["{木旁}"],
                ["號", "食", "虎"], ["號", "食", "水"],
                ["木", "口", "日"], ["⿱木林"], ["𠂉", "木"],
                ["⿰木木", "木", "木"]]:
    rust = subprocess.check_output(
        ["target/debug/hanzi-search", "find", *needles], text=True
    )
    # Lua's Rime-facing API removes duplicate characters from tagged variants.
    expected = sorted({line.split()[0] for line in rust.splitlines()})
    if not needles:
        script = ('package.path="rime/lua/?.lua;"..package.path; '
                  'local f=require("hanzi_search_find"); '
                  'for _,c in ipairs(f.find(assert(f.load("chai.txt")),{})) do print(c) end')
        cmd = ["lua5.4", "-e", script]
    else:
        cmd = ["lua5.4", "rime/tests/find.lua", *needles]
    actual = subprocess.check_output(cmd, text=True).splitlines()
    if needles:
        assert actual.pop(0) == "find unit tests passed"
    assert actual == expected, (needles, set(actual) ^ set(expected))
    print(f"parity {needles}: {len(actual)} results")
