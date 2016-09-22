bitpack
-------

```rust
let mut buff = [0; 4];

// write
{
    let mut bitpack = BitPack::<&mut [u8]>::new(&mut buff);
    bitpack.write(10, 4).unwrap();
    bitpack.write(1021, 10).unwrap();
    bitpack.write(3, 2).unwrap();
    bitpack.flush();
}

assert_eq!(buff, [0, 0, 247, 175]);

// read
{
    let mut bitpack = BitPack::<&[u8]>::new(&buff);
    assert_eq!(bitpack.read(4).unwrap(), 10);
    assert_eq!(bitpack.read(10).unwrap(), 1021);
    assert_eq!(bitpack.read(2).unwrap(), 3);
}
```
