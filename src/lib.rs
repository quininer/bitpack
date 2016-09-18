#![no_std]


#[derive(Debug, PartialEq, Eq)]
pub struct BitPack<'b> {
    buff: &'b mut [u8],
    cursor: usize
}

impl<'a> BitPack<'a> {
    pub fn new(buff: &'a mut [u8]) -> BitPack<'a> {
        BitPack {
            buff: buff,
            cursor: 0
        }
    }

    pub fn write(&mut self, value: u32, mut bits: usize) -> Result<(), ()> {
        if bits > 32 && self.cursor + bits > self.buff.len() * 8 { return Err(()) };

        loop {
            let index = self.cursor / 8;
            let width = 8 - (self.cursor % 8);
            let mask = (1 << width) - 1;

            if bits > width {
                self.buff[index] |= ((value >> (bits - width)) & mask) as u8;
                self.cursor += width;
                bits -= width;
            } else {
                self.buff[index] |= (((value & mask) << (width - bits))) as u8;
                self.cursor += bits;
                break
            }
        }
        Ok(())
    }

    pub fn read(&mut self, mut bits: usize) -> Result<u32, ()> {
        if bits > 32 && self.cursor + bits > self.buff.len() * 8 { return Err(()) };
        let mut output = 0;
        loop {
            let index = self.cursor / 8;
            let width = 8 - (self.cursor % 8);
            let mask = (1 << width) - 1;

            if bits > width {
                output |= (self.buff[index] as u32 & mask) << (bits - width);
                self.cursor += width;
                bits -= width;
            } else {
                output |= (self.buff[index] as u32 >> (width - bits)) & mask;
                self.cursor += bits;
                break
            }
        }
        Ok(output)
    }
}



#[test]
fn test_write() {
    let mut buff = [0; 2];
    let mut bitpack = BitPack::new(&mut buff);

    bitpack.write(10, 4).unwrap();
    bitpack.write(1021, 10).unwrap();
    bitpack.write(3, 2).unwrap();

    assert_eq!(bitpack.buff[0], 175);
    assert_eq!(bitpack.buff[1], 247);
}


#[test]
fn test_read() {
    let mut buff = [175, 247];
    let mut bitpack = BitPack::new(&mut buff);

    assert_eq!(bitpack.read(4).unwrap(), 10);
    assert_eq!(bitpack.read(10).unwrap(), 1021);
    assert_eq!(bitpack.read(2).unwrap(), 3);
}
