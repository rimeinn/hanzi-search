# hanzi-search：花式模糊查詢漢字

原始拆分數據來自[天碼](http://www.soongsky.com/sky/download.php)。

## find：簡單搜尋

```
cargo run --release -- find 部件1 [部件2 部件3 ...]
```

列出所有匹配這些部件的漢字，一行一个字。例如：

```
$ ./target/release/hanzi-search find 號 食
饕
```

也可以使用 IDS：

```
$ ./target/release/hanzi-search find ⿰号虎 食
饕
```

## match：全字模式匹配

```
cargo run --release -- match 模式
```

模式是 IDS，但其中的 `.` 可以用於匹配任何字符。例如：

```
$ cargo run -r -q --  match ⿲吕.吕
嘂
嚻
𡅽
𡂨
𫬽
```

該命令只能用於全字匹配，不能匹配字中的子部件。

## pmatch：部分模式匹配

```
cargo run --release -- pmatch 模式
```

用法同上，但可以用於匹配一個字中的部分組件。例如：

```
$ cargo run -r -q --  pmatch ⿲木.木
𧢣
𰙈
𣡨
𩕒
欎
𬰸
<...其餘輸出省略...>
```
