#![no_std]


const MAX_BITS: usize = 32;
const BYTE_BITS: usize = 8;

#[derive(Debug, PartialEq, Eq)]
pub struct BitPack<B> {
    pub buff: B,
    pub cursor: usize,
    pub bits: usize
}

impl<'a> BitPack<&'a mut [u8]> {
    pub fn new(buff: &mut [u8]) -> BitPack<&mut [u8]> {
        BitPack {
            buff: buff,
            cursor: 0,
            bits: 0
        }
    }

    #[inline]
    pub fn bits(&self) -> usize {
        self.cursor * BYTE_BITS + self.bits
    }

    pub fn write(&mut self, mut value: u32, mut bits: usize) -> Result<(), ()> {
        if bits > MAX_BITS || self.buff.len() * BYTE_BITS < self.bits() + bits {
            return Err(());
        }
        if bits < MAX_BITS {
            value &= (1 << bits) - 1;
        }

        loop {
            let bits_left = BYTE_BITS - self.bits;

            if bits <= bits_left {
                self.buff[self.cursor] |= (value as u8) << self.bits;
                self.bits += bits;

                if self.bits >= BYTE_BITS {
                    self.flush();
                }

                break
            }

            let vv = value & (1 << bits_left) - 1;
            self.buff[self.cursor] |= (vv as u8) << self.bits;
            self.bits += bits_left;
            value >>= bits_left;
            bits -= bits_left;

            self.flush();
        }
        Ok(())
    }

    pub fn flush(&mut self) {
        if self.bits > 0 {
            self.cursor += 1;
            self.bits = 0;
        }
    }
}


impl<'a> BitPack<&'a [u8]> {
    pub fn new(buff: &[u8]) -> BitPack<&[u8]> {
        BitPack {
            buff: buff,
            cursor: 0,
            bits: 0
        }
    }

    #[inline]
    pub fn bits(&self) -> usize {
        self.cursor * BYTE_BITS + self.bits
    }

    pub fn read(&mut self, mut bits: usize) -> Result<u32, ()> {
        if bits > MAX_BITS || self.buff.len() * BYTE_BITS < self.bits() + bits {
            return Err(());
        };

        let mut bits_left = 0;
        let mut output = 0;
        loop {
            let byte_left = BYTE_BITS - self.bits;

            if bits <= byte_left {
                let mut bb = self.buff[self.cursor] as u32;
                bb >>= self.bits;
                bb &= (1 << bits) - 1;
                output |= bb << bits_left;
                self.bits += bits;
                break
            }

            let mut bb = self.buff[self.cursor] as u32;
            bb >>= self.bits;
            bb &= (1 << byte_left) - 1;
            output |= bb << bits_left;
            self.bits += byte_left;
            bits_left += byte_left;
            bits -= byte_left;

            if self.bits >= BYTE_BITS {
                self.cursor += 1;
                self.bits -= BYTE_BITS;
            }
        }
        Ok(output)
    }
}


#[test]
fn test_bitpack() {
    let mut buff = [0; 2];

    {
        let mut bitpack = BitPack::<&mut [u8]>::new(&mut buff);
        bitpack.write(10, 4).unwrap();
        bitpack.write(1021, 10).unwrap();
        bitpack.write(3, 2).unwrap();
        bitpack.flush();
    }

    assert_eq!(buff, [218, 255]);

    {
        let mut bitpack = BitPack::<&[u8]>::new(&buff);
        assert_eq!(bitpack.read(4).unwrap(), 10);
        assert_eq!(bitpack.read(10).unwrap(), 1021);
        assert_eq!(bitpack.read(2).unwrap(), 3);
    }
}

#[test]
fn test_lowbit() {
    let mut buff = [0; 1];

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
