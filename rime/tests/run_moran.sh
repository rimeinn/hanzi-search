#!/bin/sh
set -eu
cd "$(dirname "$0")/../.."
test -f moran-dist/moran.schema.yaml
cp rime/lua/hanzi_search.lua rime/lua/hanzi_search_find.lua moran-dist/lua/
cp rime/tests/hanzi_search_driver.lua moran-dist/lua/
cp rime/tests/alternate_chai.txt moran-dist/lua/
cp chai.txt moran-dist/lua/chai.txt
cp rime/hanzi_search_moran.custom.yaml moran-dist/moran.custom.yaml
cp rime/hanzi_search_moran.custom.yaml moran-dist/
# Moran's own test target also installs the shared OpenCC resources.
for dir in /usr/share/opencc /usr/local/share/opencc; do
    if [ -d "$dir" ]; then cp -a "$dir"/. moran-dist/opencc/; fi
done
mira --shared-dir /usr/share/rime-data "$@" rime/tests/hanzi_search.test.yaml
