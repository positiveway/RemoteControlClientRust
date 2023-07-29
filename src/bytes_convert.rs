use bytes::{Buf, BytesMut};

pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

impl ToBytes for u8 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToBytes for i8 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToBytes for u16 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToBytes for i16 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToBytes for u32 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToBytes for i32 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToBytes for u64 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToBytes for i64 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToBytes for f32 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToBytes for f64 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

pub fn to_bytes<T: ToBytes, S: AsRef<[T]>>(values: S) -> Vec<u8> {
    let mut buf = vec![];
    for value in values.as_ref() {
        buf.extend(value.to_bytes());
    }
    buf
}

pub trait FromBytes<T> {
    fn from_bytes(buf: &mut BytesMut) -> T;
}

impl FromBytes<u8> for u8 {
    fn from_bytes(buf: &mut BytesMut) -> u8 {
        buf.get_u8()
    }
}

impl FromBytes<i8> for i8 {
    fn from_bytes(buf: &mut BytesMut) -> i8 {
        buf.get_i8()
    }
}

impl FromBytes<u16> for u16 {
    fn from_bytes(buf: &mut BytesMut) -> u16 {
        buf.get_u16()
    }
}

impl FromBytes<i16> for i16 {
    fn from_bytes(buf: &mut BytesMut) -> i16 {
        buf.get_i16()
    }
}

impl FromBytes<u32> for u32 {
    fn from_bytes(buf: &mut BytesMut) -> u32 {
        buf.get_u32()
    }
}

impl FromBytes<i32> for i32 {
    fn from_bytes(buf: &mut BytesMut) -> i32 {
        buf.get_i32()
    }
}

impl FromBytes<u64> for u64 {
    fn from_bytes(buf: &mut BytesMut) -> u64 {
        buf.get_u64()
    }
}

impl FromBytes<i64> for i64 {
    fn from_bytes(buf: &mut BytesMut) -> i64 {
        buf.get_i64()
    }
}

impl FromBytes<f32> for f32 {
    fn from_bytes(buf: &mut BytesMut) -> f32 {
        buf.get_f32()
    }
}

impl FromBytes<f64> for f64 {
    fn from_bytes(buf: &mut BytesMut) -> f64 {
        buf.get_f64()
    }
}

pub fn from_bytes<T: FromBytes<T>, S: AsRef<[u8]>>(buf: S) -> Vec<T> {
    let mut buf = BytesMut::from(buf.as_ref());

    let mut values: Vec<T> = vec![];
    while !buf.is_empty() {
        let converted = T::from_bytes(&mut buf);
        values.push(converted);
    }
    values
}