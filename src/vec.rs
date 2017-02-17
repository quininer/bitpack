#[cfg(all(not(feature = "use_std"), feature = "use_vec"))]
use collections::Vec;
use super::{ BYTE_BITS, BitPack };


impl Default for BitPack<Vec<u8>> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl BitPack<Vec<u8>> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self::new(Vec::with_capacity(capacity))
    }

    /// ```
    /// use bitpack::BitPack;
    ///
    /// let mut bitpack_vec = BitPack::<Vec<u8>>::with_capacity(2);
    /// bitpack_vec.write(10, 4).unwrap();
    /// bitpack_vec.write(1021, 10).unwrap();
    /// bitpack_vec.write(3, 2).unwrap();
    ///
    /// assert_eq!(bitpack_vec.as_slice(), [218, 255]);
    /// #
    /// # let mut bitpack = BitPack::<&[u8]>::new(bitpack_vec.as_slice());
    /// # assert_eq!(bitpack.read(4).unwrap(), 10);
    /// # assert_eq!(bitpack.read(10).unwrap(), 1021);
    /// # assert_eq!(bitpack.read(2).unwrap(), 3);
    /// ```
    pub fn write(&mut self, value: u32, bits: usize) -> Result<(), usize> {
        let len = self.buff.len();

        if let Some(bits) = (self.sum_bits() + bits).checked_sub(len * BYTE_BITS) {
            self.buff.resize(len + (bits + BYTE_BITS - 1) / BYTE_BITS, 0x0);
        }

        let mut bitpack = BitPack {
            buff: self.buff.as_mut_slice(),
            cursor: self.cursor,
            bits: self.bits
        };

        bitpack.write(value, bits)?;

        self.bits = bitpack.bits;
        self.cursor = bitpack.cursor;

        Ok(())
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buff
    }
}


#[test]
fn test_lowbit() {
    let mut bitpack_vec = BitPack::<Vec<u8>>::with_capacity(1);
    bitpack_vec.write(1, 1).unwrap();
    bitpack_vec.write(0, 1).unwrap();
    bitpack_vec.write(0, 1).unwrap();
    bitpack_vec.write(1, 1).unwrap();

    let mut bitpack = BitPack::<&[u8]>::new(bitpack_vec.as_slice());
    assert_eq!(bitpack.read(1).unwrap(), 1);
    assert_eq!(bitpack.read(1).unwrap(), 0);
    assert_eq!(bitpack.read(1).unwrap(), 0);
    assert_eq!(bitpack.read(1).unwrap(), 1);
}

#[test]
fn test_bigbit() {
    let mut bitpack_vec = BitPack::<Vec<u8>>::with_capacity(8);
    bitpack_vec.write(255, 8).unwrap();
    bitpack_vec.write(65535, 16).unwrap();
    bitpack_vec.write(65535, 16).unwrap();
    bitpack_vec.write(255, 8).unwrap();
    bitpack_vec.write(65535, 16).unwrap();

    let mut bitpack = BitPack::<&[u8]>::new(bitpack_vec.as_slice());
    assert_eq!(bitpack.read(8).unwrap(), 255);
    assert_eq!(bitpack.read(16).unwrap(), 65535);
    assert_eq!(bitpack.read(16).unwrap(), 65535);
    assert_eq!(bitpack.read(8).unwrap(), 255);
    assert_eq!(bitpack.read(16).unwrap(), 65535);
}

#[test]
fn test_morelowbit() {
    let input = [
        1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0,
        1, 1, 1, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1,
        0, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1,
    ];

    let mut bitpack_vec = BitPack::<Vec<u8>>::with_capacity(8);
    for &b in &input[..] {
        bitpack_vec.write(b, 1).unwrap();
    }

    let mut bitpack = BitPack::<&[u8]>::new(bitpack_vec.as_slice());
    for &b in &input[..] {
        assert_eq!(bitpack.read(1).unwrap(), b);
    }
}
