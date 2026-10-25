# Rime 拆字查詢

⚠️ Lua 腳本爲 100% Vibe Code。原算法參考自 hanzi-search。

基於 librime-lua 的 Rime 拆字查詢插件。使用當前方案輸入部件，查找包含這些部件的漢字。

## 安裝

1. 將 `rime/lua/hanzi_search.lua` 和 `rime/lua/hanzi_search_find.lua` 複製到 Rime 用戶目錄的 `lua/`。
2. 將本倉庫的 `chai.txt` 複製到 Rime 用戶目錄的 `lua/chai.txt`。
3. 將下列配置合併到所用方案的 `<schema_id>.custom.yaml`（例如 `moran.custom.yaml`）。
4. 重新部署 Rime，切換到該方案。

```yaml
patch:
  engine/processors/@before 0: lua_processor@*hanzi_search*editor
  engine/segmentors/@before 0: lua_segmentor@*hanzi_search*segmentor
  engine/translators/@before 0: lua_translator@*hanzi_search*translator
  engine/filters/@before 0: lua_filter@*hanzi_search*filter
```

## 使用

輸入 `oi`，再按當前方案的編碼輸入部件並選字。每確認一個部件，候選欄就會顯示查詢結果；繼續打碼可添加下一個部件，選擇查詢結果即可上屏。

以魔然爲例：`oi` → `hkh` → 選「號」→ `uirl` → 選「食」→ 選「饕」。

- **空格、回車、數字或點選**：確認候選。
- **退格**：刪除輸入；當前段爲空時，重新編輯上一部件。
- **Escape**：取消查詢。

## 自定義

在所用方案的 `.custom.yaml` 的 `patch` 中修改引導碼和數據路徑：

```yaml
patch:
  hanzi_search/prefix: oi
  hanzi_search/data_file: lua/chai.txt
```

數據路徑相對於 Rime 用戶目錄。修改配置後重新部署；更新數據文件後重啓 Rime 前端。

## 測試

在本倉庫根目錄執行：

```bash
lua5.4 rime/tests/find.lua
lua5.4 rime/tests/cache.lua
lua5.4 rime/tests/lazy.lua
python3 rime/tests/parity.py
```

使用魔然運行 Mira 集成測試：

```bash
make -C ~/src/Clone/plum/package/rimeinn/moran dist DESTDIR="$PWD/moran-dist"
sh rime/tests/run_moran.sh --cache-dir /tmp/hanzi-search-mira-cache
```
