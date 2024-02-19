# Streams

Rust library for handling [`Write`](https://doc.rust-lang.org/std/io/trait.Write.html) and [`Read`](https://doc.rust-lang.org/std/io/trait.Read.html) streams.

## Developer Notes

Possible alternative implementation of `MultiWriter::write`:

```rust
/// Write a buffer into each internal writer.
///
/// The returned `usize` will always be exactly the length of the input buffer (`buf.len()`). See [`MultiWriter`] for more information.
fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let mut writtens = vec![0usize; self.writers.len()];

    loop {
        for (i, writer) in self.writers.iter_mut().enumerate() {
            let written = writtens[i];

            if written == buf.len() {
                continue;
            }

            writtens[i] += writer.write(&buf[written..])?;
        }

        if writtens.iter().all(|&w| w == buf.len()) {
            break;
        }
    }

    Ok(buf.len())
}
```

Instead of blocking on each writer in sequence by calling `Write::write_all`, this uses `Write::write` which lets each writer signal that the operation is blocking. We can therefore skip this writer and return to it later.

If one writer is remaining but continously signals the write would be blocking, this implementation would busy-block and probably use 100% CPU by calling `write` in a loop.
