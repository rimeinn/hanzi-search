# hanzi-search：花式模糊查詢漢字

原始拆分數據來自[天碼](http://www.soongsky.com/sky/download.php)。

在線使用 [Web demo](https://ksqsf.github.io/) 。Demo 版本可能比較舊，請以本 repo 爲準。

## 使用方式

### CLI (命令行版)

#### Build

```bash
cargo build --release
```

The binary is self-contained with the character database embedded - no external files needed.

#### find：簡單搜尋

```bash
cargo run --release -- find 部件1 [部件2 部件3 ...]
```

列出所有匹配這些部件的漢字，一行一个字。例如：

```bash
$ ./target/release/hanzi-search find 號 食
饕
```

也可以使用 IDS：

```bash
$ ./target/release/hanzi-search find ⿰号虎 食
饕
```

#### match：全字模式匹配

```bash
cargo run --release -- match 模式
```

模式是 IDS，但其中的 `.` 可以用於匹配任何字符。例如：

```bash
$ cargo run -r -q -- match ⿲吕.吕
嘂
嚻
𡅽
𡂨
𫬽
```

該命令只能用於全字匹配，不能匹配字中的子部件。

#### pmatch：部分模式匹配

```bash
cargo run --release -- pmatch 模式
```

用法同上，但可以用於匹配一個字中的部分組件。例如：

```bash
$ cargo run -r -q -- pmatch ⿲木.木
𧢣
𰙈
𣡨
𩕒
欎
𬰸
<...其餘輸出省略...>
```

### Web 界面

1. 構建 wasm，或直接在 Release 頁面下載構建產物
2. 啓動 web 服務器
   ```bash
   python3 -m http.server 8000
   ```
3. 用瀏覽器打開 http://localhost:8000

## 開發

### 構建 CLI

```bash
cargo build --release
```

### 構建 WASM

```bash
wasm-pack build --target web
```

