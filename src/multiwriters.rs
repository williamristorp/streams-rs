use std::io::{self, Read, Write};

#[derive(Default)]
pub struct MultiWriter<'a> {
    writers: Vec<&'a mut dyn Write>,
}

impl<'a> Write for MultiWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for writer in &mut self.writers {
            writer.write_all(buf)?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        for writer in &mut self.writers {
            writer.flush()?;
        }

        Ok(())
    }
}

pub fn copy_many<R: Read + ?Sized>(
    reader: &mut R,
    writers: Vec<&mut dyn Write>,
) -> io::Result<u64> {
    let mut multi_writer = MultiWriter { writers };
    io::copy(reader, &mut multi_writer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_writer() {
        let mut writers = vec![Vec::<u8>::new(), Vec::new(), Vec::new()];
        let mut multi_writer = MultiWriter {
            writers: writers.iter_mut().map(|o| o as &mut dyn Write).collect(),
        };

        let input = b"Hello, world!";
        multi_writer.write_all(input).unwrap();

        for output in writers {
            assert_eq!(output[..], *b"Hello, world!");
        }
    }

    #[test]
    fn copy_many_vec() {
        let input = b"Hello, world!";
        let mut outputs = vec![Vec::<u8>::new(), Vec::new(), Vec::new()];
        copy_many(
            &mut &input[..],
            outputs.iter_mut().map(|o| o as &mut dyn Write).collect(),
        )
        .unwrap();
    }
}
