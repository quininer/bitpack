bitpack
-------

```rust
let mut buff = [0; 2];

// write
{
    let mut bitpack = BitPack::<&mut [u8]>::new(&mut buff);
    bitpack.write(10, 4).unwrap();
    bitpack.write(1021, 10).unwrap();
    bitpack.write(3, 2).unwrap();
}

assert_eq!(buff, [218, 255]);

// read
{
    let mut bitpack = BitPack::<&[u8]>::new(&buff);
    assert_eq!(bitpack.read(4).unwrap(), 10);
    assert_eq!(bitpack.read(10).unwrap(), 1021);
    assert_eq!(bitpack.read(2).unwrap(), 3);
}
```

and, `use_std`

```rust
let mut bitpack_vec = BitPack::<Vec<u8>>::with_capacity(2);
bitpack_vec.write(10, 4).unwrap();
bitpack_vec.write(1021, 10).unwrap();
bitpack_vec.write(3, 2).unwrap();

assert_eq!(bitpack_vec.as_slice(), [218, 255]);

let mut bitpack = BitPack::<&[u8]>::new(bitpack_vec.as_slice());
assert_eq!(bitpack.read(4).unwrap(), 10);
assert_eq!(bitpack.read(10).unwrap(), 1021);
assert_eq!(bitpack.read(2).unwrap(), 3);
```
