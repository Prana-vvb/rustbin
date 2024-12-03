# rustbin
```text
 ___/-\___
|---------|
 |   |   |
 | | | | |     Rustbin
 | | | | |     A simple pastebin written in Rust🦀
 | | | | |
 |_______|
```
Inspired by [polarhive/pasta](https://github.com/polarhive/pasta)

## Install
```
git clone --depth 1 https://github.com/Prana-vvb/rustbin.git
```

CD to the clone and build using `Cargo`
```
cd rustbin
cargo build --release
```

And run the binary in `target/release`

Or run an unoptimized version with
```
cd rustbin
cargo run
```

## Usage
Paste a file:
```
curl -F "file=@filename" "localhost:8080/data"
```

Retrieve a file:
```
curl "localhost:8080/data/<UID of paste>"
```

## References
- [What is a pastebin?](https://systemdesign.one/system-design-pastebin)
- [Pasta](https://github.com/polarhive/pasta)
