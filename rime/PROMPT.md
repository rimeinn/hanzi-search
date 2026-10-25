use(rime skills);
reference(rime-moran, at=~/src/Clone/plum/package/rimeinn/moran/);
target = librime_lua;
strategy = hanzi_search_find;
data_location = "lua/chai.txt";
expose_as = [lua_translator, lua_filter, lua_editor, lua_segmentor];
logic = {
  activate on input.startswith("oi");
  each component is input by a segment:
    if a segment is not confirmed:
      call the main translator and generate candidates
  on confirmation of a candidate from component candidates:
    push the selected component string
    push an empty segment
  on empty segment:
    yield from hanzi_search_find(currently confirmed components)
  on confirmation of a candidate from hanzi_search_find:
    commit and finish translation
}
