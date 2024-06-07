
# mμnzip

**mμnzip** is a port of [JUnzip](https://github.com/jokkebk/JUnzip) to Rust.

While **JUnzip** supports `STORE`, and optionally `DEFLATE` and/or `ZLIB`, **mμnzip** foregoes `ZLIB` support.

### features

[iterable](examples/iterate.rs)

```rust
let zi = munzip::IterableArchive::new(&mut input).unwrap();

for entry in zi {
    let mut entry = entry.unwrap();
    let filename = entry.filename();
    let buffer = entry.buffer().unwrap();
    write::write_file(&filename, &buffer).unwrap();
}
```

[searchable](examples/search.rs)

```rust
let mut zi = munzip::SearchableArchive::new(&mut input).unwrap();

let filename = "munzip/Cargo.toml";
let cargo_toml = zi.by_name(filename).unwrap().unwrap();
write::write_file(&"Cargo.toml".to_owned(), &cargo_toml).unwrap();
```

### why?

- To be small.
- To have a simple unzip library with minimal (2)([inflate](https://crates.io/crates/inflate),[adler32](https://crates.io/crates/adler32/1.2.0)) dependencies.
- For fun and practice.

### stats

| Method  | Dependencies | Size | Speed   |
| ------- | ------------ | ---- | ------- |
| mμnzip  | 2            | 419K | 0.327s  |
| zip     | 27           | 491K | 0.158s  |
| unzip   | 2            | 2.1M | 0.286s  |

CONCLUSIONS:

- is it very fast? no
- is it very small? no
- very featureful? no

😐👍

NOTES:

- 'zip' refers to the [zip](https://crates.io/crates/zip) crate
- 'zip' was set to `default-features = false, features = ["deflate"]`
- 'unzip' refers to debian's [unzip](https://packages.debian.org/bookworm/unzip) package
- 'unzip' size includes libc and libbz2
- all binaries are stripped
- `Speed` was measured by unzipping an archive of this entire repository on my laptop, so.