#![cfg_attr(not(feature = "use_std"), no_std)]

#[cfg(feature = "use_std")]
mod vec;


pub const MAX_BITS: usize = 32;
const BYTE_BITS: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitPack<B> {
    buff: B,
    cursor: usize,
    bits: usize
}

impl<B> BitPack<B> {
    #[inline]
    pub fn new(buff: B) -> Self {
        BitPack { buff: buff, cursor: 0, bits: 0 }
    }

    #[inline]
    pub fn sum_bits(&self) -> usize {
        self.cursor * BYTE_BITS + self.bits
    }

    #[inline]
    pub fn with_cursor(&mut self, cursor: usize) -> &mut Self {
        self.cursor = cursor;
        self
    }

    #[inline]
    pub fn with_bits(&mut self, bits: usize) -> &mut Self {
        self.bits = bits;
        self
    }
}

impl<B: AsRef<[u8]>> BitPack<B> {
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.buff.as_ref()
    }
}

impl<'a> BitPack<&'a mut [u8]> {
    /// ```
    /// use bitpack::BitPack;
    ///
    /// let mut buff = [0; 2];
    ///
    /// {
    ///     let mut bitpack = BitPack::<&mut [u8]>::new(&mut buff);
    ///     bitpack.write(10, 4).unwrap();
    ///     bitpack.write(1021, 10).unwrap();
    ///     bitpack.write(3, 2).unwrap();
    /// }
    ///
    /// assert_eq!(buff, [218, 255]);
    /// ```
    pub fn write(&mut self, mut value: u32, mut bits: usize) -> Result<(), usize> {
        if bits > MAX_BITS || self.buff.len() * BYTE_BITS < self.sum_bits() + bits {
            return Err(bits);
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
                    self.cursor += 1;
                    self.bits = 0;
                }

                break
            }

            let bb = value & ((1 << bits_left) - 1);
            self.buff[self.cursor] |= (bb as u8) << self.bits;
            self.cursor += 1;
            self.bits = 0;
            value >>= bits_left;
            bits -= bits_left;
        }
        Ok(())
    }
}


impl<'a> BitPack<&'a [u8]> {
    /// ```
    /// use bitpack::BitPack;
    ///
    /// let mut buff = [218, 255];
    ///
    /// let mut bitpack = BitPack::<&[u8]>::new(&buff);
    /// assert_eq!(bitpack.read(4).unwrap(), 10);
    /// assert_eq!(bitpack.read(10).unwrap(), 1021);
    /// assert_eq!(bitpack.read(2).unwrap(), 3);
    /// ```
    pub fn read(&mut self, mut bits: usize) -> Result<u32, usize> {
        if bits > MAX_BITS || self.buff.len() * BYTE_BITS < self.sum_bits() + bits {
            return Err(bits);
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
fn test_smallbit() {
    let mut buff = [0; 1];

    {
        let mut bitpack = BitPack::<&mut [u8]>::new(&mut buff);
        bitpack.write(1, 1).unwrap();
        bitpack.write(0, 1).unwrap();
        bitpack.write(0, 1).unwrap();
        bitpack.write(1, 1).unwrap();
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
fn test_moresmallbit() {
    let input = [
        1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0,
        1, 1, 1, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1,
        0, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1,
    ];
    let mut buff = [0; 8];

    {
        let mut bitpack = BitPack::<&mut [u8]>::new(&mut buff);
        for &b in &input[..] {
            bitpack.write(b, 1).unwrap();
        }
    }

    {
        let mut bitpack = BitPack::<&[u8]>::new(&buff);
        for &b in &input[..] {
            assert_eq!(bitpack.read(1).unwrap(), b);
        }
    }
}
