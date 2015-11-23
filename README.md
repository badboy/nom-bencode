# Bencode parser

[Bencode][bencoding] is the format used in Bittorrent files.
It's a very simplistic format.
All it knows about are strings, integers, lists and dictionaries (also known as hash maps).

Using [nom][] I wrote a small parser able to parse all this.

```rust
let data = "l5:jelly4:cake7:custarde".as_bytes();
let obj = bencode::value(data);
```

[bencoding]: https://wiki.theory.org/BitTorrentSpecification#Bencoding
[nom]: https://github.com/Geal/nom
