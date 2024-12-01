# rustbin
```text
 ___/-\___
|---------|
 |   |   |
 | | | | |     Rustbin
 | | | | |     A simple pastebin written in Rust  
 | | | | |
 |_______|
```
## Install
```
git clone --depth 1 https://github.com/Prana-vvb/rustbin.git
```

CD to the clone and start the server using `Cargo`
```
cd rustbin
cargo run
```

## Usage
Paste a file:
```
curl -F "file=@file.txt" "localhost:8080/data"
```

Retrieve a file:
```
curl "localhost:8080/data/<UID of paste>"
```
