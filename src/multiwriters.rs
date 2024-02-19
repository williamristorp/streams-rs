use std::io::{self, Read, Write};

/// Provides a single [`Writer`](Write) that writes to multiple writers sequentially.
///
/// The [`write`](MultiWriter::write) implementation calls [`write_all`](Write::write_all) on each of the internal writers in sequence
/// and therefore behaves as a call to [`write_all`](Write::write_all) itself;
/// in fact, [`MultiWriter::write_all`] simply calls this function, discarding the returned `usize`
/// (which will always be exactly the length of the input buffer (`buf.len()`)).
///
/// # Errors
///
/// If any of the internal writers fail during the iteration,
/// execution will immediately halt and the error will be returned.
/// Currently, this implementation provides no method of determining which writer failed.
///
/// Keep in mind that some of the internal writes may have been succesfully executed even if a following write fails.
///
/// # Blocking
///
/// `MultiWriter` is blocking to the extent that its internal writers are blocking.
///
/// # Implementation Notes
///
/// As we can only return a single `usize`,
/// and each internal writer may return differing numbers of bytes from their respective [`write`](Write::write) call,
/// it is not trivial how one could avoid using [`write_all`](Write::write_all).
///
/// One alternative would be to call [`write`](Write::write) on the first internal writer,
/// then use its returned `usize` and call [`write_all`](Write::write_all) on the remaining writers,
/// writing only the same number of bytes as the first.
/// To avoid assigning arbitary and non-obvious meaning to the order of the internal writers [`Vec`],
/// such an implementation should consider adopting a master-slaves pattern and make it obvious that the first writer's result will impact the others.
pub struct MultiWriter<'a> {
    writers: Vec<&'a mut dyn Write>,
}

impl<'a> Write for MultiWriter<'a> {
    /// Write a buffer into each internal writer sequentially.
    ///
    /// The returned `usize` will always be exactly the length of the input buffer (`buf.len()`). See [`MultiWriter`] for more information.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for writer in &mut self.writers {
            writer.write_all(buf)?;
        }

        Ok(buf.len())
    }

    /// Flush each internal output stream sequentially, ensuring that all intermediately buffered contents reach their destinations.
    fn flush(&mut self) -> io::Result<()> {
        for writer in &mut self.writers {
            writer.flush()?;
        }

        Ok(())
    }

    /// Calls [`write`](MultiWriter::write) and discards the returned `usize`.
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        // Explicitely drop the returned `usize` to make `cargo clippy` happy.
        let _ = self.write(buf)?;

        Ok(())
    }
}

/// Copy the entire contents of a reader into multiple writers.
///
/// Uses a [`MultiWriter`] and [`std::io::copy`].
pub fn copy_into_many<R: Read + ?Sized>(
    reader: &mut R,
    writers: Vec<&mut dyn Write>,
) -> io::Result<u64> {
    let mut multi_writer = MultiWriter { writers };
    io::copy(reader, &mut multi_writer)
}

/// Utility macro to avoid manually casting writers to `&mut dyn std::io::Write`.
#[macro_export]
macro_rules! copy_into_many {
    ($reader:expr,$writers:expr) => {{
        $crate::copy_into_many(
            $reader,
            $writers
                .iter_mut()
                .map(|o| o as &mut dyn std::io::Write)
                .collect(),
        )
    }};
}

/// Utility macro to avoid collecting writers into a `Vec<&mut dyn std::io::Write>`.
#[macro_export]
macro_rules! copy_into_all {
    ($reader:expr,$($writer:expr),*) => {{
        let writers = vec![$(&mut $writer as &mut dyn std::io::Write,)*];
        $crate::copy_into_many(
            $reader,
            writers,
        )
    }};
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        io::{Cursor, Write},
    };

    #[test]
    fn multi_writer() {
        let mut writers = vec![Vec::<u8>::new(), Vec::new(), Vec::new()];
        let mut multi_writer = crate::MultiWriter {
            writers: writers.iter_mut().map(|o| o as &mut dyn Write).collect(),
        };

        let input = b"Hello, world!";
        multi_writer.write_all(input).unwrap();

        for writer in writers {
            assert_eq!(writer[..], *b"Hello, world!");
        }
    }

    #[test]
    fn copy_into_many_vec() {
        let input = b"Hello, world!";
        let mut writers = vec![Vec::<u8>::new(), Vec::new(), Vec::new()];

        crate::copy_into_many(
            &mut &input[..],
            writers.iter_mut().map(|o| o as &mut dyn Write).collect(),
        )
        .unwrap();

        for writer in writers {
            assert_eq!(writer[..], *b"Hello, world!");
        }
    }

    #[test]
    fn copy_into_many_macro() {
        let input = b"Hello, world!";
        let mut writers = vec![Vec::<u8>::new(), Vec::new(), Vec::new()];

        crate::copy_into_many!(&mut &input[..], writers).unwrap();

        for writer in writers {
            assert_eq!(writer[..], *b"Hello, world!");
        }
    }

    #[test]
    fn copy_into_all_macro() {
        let input = b"Hello, world!";
        let mut writer1 = Vec::new();
        let mut writer2 = Cursor::new(Vec::new());
        let mut writer3 = VecDeque::new();

        crate::copy_into_all!(&mut &input[..], writer1, writer2, writer3).unwrap();

        assert_eq!(writer1, *b"Hello, world!");
        assert_eq!(writer2.into_inner(), *b"Hello, world!");
        assert_eq!(writer3, *b"Hello, world!");
    }
}
