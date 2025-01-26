# nasubot

nasubotはNAS用PCの健康状態をSlackに通知する。
次を通知する。

- ディスクの残量
- `btrfs scrub status`の実行結果

## Ubuntuへのクロスコンパイル方法

dockerを実行後に次のコマンドを実行する。

```sh
cross build --target x86_64-unknown-linux-musl --release
```
