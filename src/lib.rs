#![no_std]

extern crate byteorder;

use byteorder::{ LittleEndian, ByteOrder };


pub const BUF_BITS: usize = 32;
const BYTE_BITS: usize = 8;

#[derive(Debug, PartialEq, Eq)]
pub struct BitPack<B> {
    pub buff: B,
    pub cursor: usize,
    pub bits_left: usize,
    pub bits_buf: u32,
    pub bits: usize
}

impl<'a> BitPack<&'a mut [u8]> {
    pub fn new(buff: &mut [u8]) -> BitPack<&mut [u8]> {
        assert_eq!(buff.len() % (BUF_BITS / BYTE_BITS), 0);
        BitPack {
            buff: buff,
            bits_left: BUF_BITS,
            bits_buf: 0,
            bits: 0,
            cursor: 0
        }
    }

    pub fn write(&mut self, mut value: u32, mut bits: usize) -> Result<(), ()> {
        if bits > BUF_BITS { return Err(()) };
        if bits < BUF_BITS {
            value &= (1 << bits) - 1;
        }
        if self.buff.len() * BYTE_BITS < self.bits + bits { return Err(()) };
        self.bits += bits;

        loop {
            if bits <= self.bits_left {
                self.bits_buf |= value << (self.bits_left - bits);
                self.bits_left -= bits;
                break
            }

            self.bits_buf |= value >> (bits - self.bits_left);
            value &= (1 << (bits - self.bits_left)) - 1;
            bits -= self.bits_left;

            self.flush();
        }

        Ok(())
    }

    pub fn flush(&mut self) {
        LittleEndian::write_u32(&mut self.buff[self.cursor..], self.bits_buf);
        self.cursor += BUF_BITS / BYTE_BITS;
        self.bits_buf = 0;
        self.bits_left = BUF_BITS;
    }
}

impl<'a> BitPack<&'a [u8]> {
    pub fn new(buff: &[u8]) -> BitPack<&[u8]> {
        assert_eq!(buff.len() % (BUF_BITS / BYTE_BITS), 0);
        BitPack {
            buff: buff,
            bits_left: 0,
            bits_buf: 0,
            bits: 0,
            cursor: 0
        }
    }

    pub fn read(&mut self, mut bits: usize) -> Result<u32, ()> {
        if bits > BUF_BITS { return Err(()) };
        if self.buff.len() * BYTE_BITS < self.bits + bits { return Err(()) };
        let mut output = 0;
        loop {
            if self.bits_left == 0 {
                self.bits_buf = LittleEndian::read_u32(&self.buff[self.cursor..]);
                self.cursor += BUF_BITS / BYTE_BITS;
                self.bits_left = BUF_BITS;
            }
            if bits <= self.bits_left {
                output |= self.bits_buf >> (self.bits_left - bits);
                self.bits_buf &= (1 << (self.bits_left - bits)) - 1;
                self.bits_left -= bits;
                break
            }
            output |= self.bits_buf << (bits - self.bits_left);
            bits -= self.bits_left;
            self.bits_left = 0;
        }
        Ok(output)
    }
}


#[test]
fn test_bitpack() {
    let mut buff = [0; 4];

    {
        let mut bitpack = BitPack::<&mut [u8]>::new(&mut buff);
        bitpack.write(10, 4).unwrap();
        bitpack.write(1021, 10).unwrap();
        bitpack.write(3, 2).unwrap();
        bitpack.flush();
    }

    assert_eq!(buff, [0, 0, 247, 175]);

    {
        let mut bitpack = BitPack::<&[u8]>::new(&buff);
        assert_eq!(bitpack.read(4).unwrap(), 10);
        assert_eq!(bitpack.read(10).unwrap(), 1021);
        assert_eq!(bitpack.read(2).unwrap(), 3);
    }
}

#[test]
fn test_lowbit() {
    let mut buff = [0; 4];

    {
        let mut bitpack = BitPack::<&mut [u8]>::new(&mut buff);
        bitpack.write(1, 1).unwrap();
        bitpack.write(0, 1).unwrap();
        bitpack.write(0, 1).unwrap();
        bitpack.write(1, 1).unwrap();
        bitpack.flush();
    }

    {
        let mut bitpack = BitPack::<&[u8]>::new(&buff);
        assert_eq!(bitpack.read(1).unwrap(), 1);
        assert_eq!(bitpack.read(1).unwrap(), 0);
        assert_eq!(bitpack.read(1).unwrap(), 0);
        assert_eq!(bitpack.read(1).unwrap(), 1);
    }
}

#[test]
fn test_bigbit() {
    let mut buff = [0; 8];

    {
        let mut bitpack = BitPack::<&mut [u8]>::new(&mut buff);
        bitpack.write(255, 8).unwrap();
        bitpack.write(65535, 16).unwrap();
        bitpack.write(65535, 16).unwrap();
        bitpack.write(255, 8).unwrap();
        bitpack.write(65535, 16).unwrap();
        bitpack.flush();
    }

    {
        let mut bitpack = BitPack::<&[u8]>::new(&buff);
        assert_eq!(bitpack.read(8).unwrap(), 255);
        assert_eq!(bitpack.read(16).unwrap(), 65535);
        assert_eq!(bitpack.read(16).unwrap(), 65535);
        assert_eq!(bitpack.read(8).unwrap(), 255);
        assert_eq!(bitpack.read(16).unwrap(), 65535);
    }
}

#[test]
fn test_longlowbit() {
    let input = [
        1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0,
        1, 1, 1, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1,
        0, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let mut buff = [0; 8];

    {
        let mut bitpack = BitPack::<&mut [u8]>::new(&mut buff);
        for &b in &input[..] {
            bitpack.write(b, 1).unwrap();
        }
        bitpack.flush();
    }

    {
        let mut bitpack = BitPack::<&[u8]>::new(&buff);
        for &b in &input[..] {
            assert_eq!(bitpack.read(1).unwrap(), b);
        }
    }
}
