use std::io::{self, Read, Write};

pub struct RoundRobinCopier<'a> {
    writers: Vec<&'a mut dyn Write>,
    current: usize,
}

impl<'a> RoundRobinCopier<'a> {
    pub fn new(writers: Vec<&'a mut dyn Write>) -> Self {
        let current = 0;

        Self { writers, current }
    }

    pub fn copy<R: Read + ?Sized>(&mut self, reader: &mut R) -> io::Result<u64> {
        let index = self.current;

        // Increment the current index, wrapping around if we exceed the number of internal writers.
        self.current = (self.current + 1) % self.writers.len();

        io::copy(reader, self.writers[index])
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::RoundRobinCopier;

    #[test]
    fn round_robin_copier() {
        let mut writers = vec![Vec::<u8>::new(), Vec::new(), Vec::new()];
        let mut copier =
            RoundRobinCopier::new(writers.iter_mut().map(|w| w as &mut dyn Write).collect());

        let input = b"Hello, world!";

        copier.copy(&mut &input[..]).unwrap();
        copier.copy(&mut &input[..]).unwrap();
        copier.copy(&mut &input[..]).unwrap();
        copier.copy(&mut &input[..]).unwrap();

        assert_eq!(writers[0], b"Hello, world!Hello, world!");
        assert_eq!(writers[1], b"Hello, world!");
        assert_eq!(writers[2], b"Hello, world!");
    }
}
