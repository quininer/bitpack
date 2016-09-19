#![no_std]

extern crate byteorder;

use byteorder::{ LittleEndian, ByteOrder };


#[derive(Debug, PartialEq, Eq)]
pub struct BitPack<'b> {
    buff: &'b mut [u8],
    bits_left: usize,
    bits_buf: u32,
    bits: usize,
    cursor: usize
}

impl<'a> BitPack<'a> {
    pub fn new(buff: &'a mut [u8]) -> BitPack<'a> {
        let mut bitpack = BitPack::from(buff);
        bitpack.bits_left = 32;
        bitpack
    }

    pub fn from(buff: &'a mut [u8]) -> BitPack<'a> {
        BitPack {
            buff: buff,
            bits_left: 0,
            bits_buf: 0,
            bits: 0,
            cursor: 0
        }
    }

    pub fn write(&mut self, mut value: u32, mut bits: usize) -> Result<(), ()> {
        if bits > 32 { return Err(()) };
        if bits < 32 {
            value &= (1 << bits) - 1;
        }
        if self.buff.len() * 8 < self.bits + bits {
            return Err(());
        } else {
            self.bits += bits;
        }

        loop {
            if bits <= self.bits_left {
                self.bits_buf |= value << (self.bits_left - bits);
                self.bits_left -= bits;
                break
            }

            self.bits_buf |= value << (bits - self.bits_left);
            value &= (1 << (bits - self.bits_left)) - 1;
            bits -= self.bits_left;

            self.flush();
        }

        Ok(())
    }

    pub fn read(&mut self, mut bits: usize) -> Result<u32, ()> {
        if bits > 32 { return Err(()) };
        if self.buff.len() * 8 < self.bits + bits { return Err(()) };
        let mut output = 0;
        loop {
            if self.bits_left == 0 {
                self.bits_buf = LittleEndian::read_u32(&self.buff[self.cursor..]);
                self.cursor += 4;
                self.bits_left = 32;
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

    pub fn flush(&mut self) {
        LittleEndian::write_u32(&mut self.buff[self.cursor..], self.bits_buf);
        self.cursor += 4;
        self.bits_buf = 0;
        self.bits_left = 32;
    }
}


#[test]
fn test_bitpack() {
    let mut buff = [0; 4];

    {
        let mut bitpack = BitPack::new(&mut buff);

        bitpack.write(10, 4).unwrap();
        bitpack.write(1021, 10).unwrap();
        bitpack.write(3, 2).unwrap();
        bitpack.flush();
    }

    {
        let mut bitpack = BitPack::from(&mut buff);

        bitpack.bits_left = 0;
        assert_eq!(bitpack.read(4).unwrap(), 10);
        assert_eq!(bitpack.read(10).unwrap(), 1021);
        assert_eq!(bitpack.read(2).unwrap(), 3);
    }
}

#[test]
fn test_lowbit() {
    let mut buff = [0; 4];

    {
        let mut bitpack = BitPack::new(&mut buff);
        bitpack.write(1, 1).unwrap();
        bitpack.write(0, 1).unwrap();
        bitpack.write(0, 1).unwrap();
        bitpack.flush();
    }

    {
        let mut bitpack = BitPack::from(&mut buff);
        assert_eq!(bitpack.read(1).unwrap(), 1);
        assert_eq!(bitpack.read(1).unwrap(), 0);
        assert_eq!(bitpack.read(1).unwrap(), 0);
    }
}
