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

## Functional Requirements (As per [System Design](https://systemdesign.one/system-design-pastebin))
- [x] ~Online text storage service similar to pastebin.com or GitHub gist~
- [x] ~A client (user) enters text data into the system known as a paste~
- [x] ~A paste must not be greater than 1 MB in size~
- [x] ~The system must return a unique paste ID for each Paste~
- [x] ~The client visiting the paste ID must be able to view the paste~
- [ ] ~The system must support only text-based data for a paste~ - WON'T IMPLEMENT
- [x] ~The paste ID should be readable~
- [x] ~The paste ID should be collision-free~
- [x] ~The paste ID should be non-predictable~
- [ ] The client should be able to choose a custom paste ID
- [ ] The paste ID should generate an analytics report (not real-time) such as the total number of access to a paste
- [ ] The client should be able to define the expiration time of the paste
- [ ] The client should be able to delete a paste
- [ ] The client must be able to set the visibility of the paste (public, private)
- [ ] The client must be able to set an optional password for the paste
- [ ] A paste must be filtered by Pastebin to prevent questionable content

## References
- [What is a pastebin?](https://systemdesign.one/system-design-pastebin)
- [Pasta](https://github.com/polarhive/pasta)
