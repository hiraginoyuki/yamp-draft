use std::{time::{Instant, Duration}, io::{Read, self}};
use mc_varint::VarIntRead;

pub trait Timer {
    fn reset(&mut self) -> &Instant;
    fn get_elapsed_then_reset(&mut self) -> Duration;
}

impl Timer for Instant {
    fn reset(&mut self) -> &Instant {
        *self = Instant::now();
        self
    }

    fn get_elapsed_then_reset(&mut self) -> Duration {
        let e = self.elapsed();
        self.reset();
        e
    }
}

pub trait ByteRead {
    fn read_bytes(&mut self, size: usize) -> io::Result<Vec<u8>>;
    fn read_byte(&mut self) -> io::Result<u8>;
}
impl<R> ByteRead for R where R: Read {
    fn read_bytes(&mut self, size: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0u8; size];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
    fn read_byte(&mut self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

pub trait McRead {
    fn read_mc_string(&mut self) -> io::Result<Vec<u8>>;
}
impl<R> McRead for R where R: io::Read {
    fn read_mc_string(&mut self) -> io::Result<Vec<u8>> {
        let length: usize = i32::from(self.read_var_int()?).try_into().unwrap();
        self.read_bytes(length)
    }
}
