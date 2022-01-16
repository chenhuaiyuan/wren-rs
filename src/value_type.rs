use std::collections::HashMap;
pub enum Number {
    PosInt(u64),
    NegInt(i64),
    Float(f64),
}
impl Number {
    #[inline]
    pub fn is_i64(&self) -> bool {
        match self {
            Number::PosInt(v) => v <= &(i64::MAX as u64),
            Number::NegInt(_) => true,
            Number::Float(_) => false,
        }
    }

    #[inline]
    pub fn is_u64(&self) -> bool {
        matches!(self, Number::PosInt(_))
    }
    #[inline]
    pub fn is_f64(&self) -> bool {
        matches!(self, Number::Float(_))
    }
}

macro_rules! impl_from_unsigned {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Number {
                #[inline]
                fn from(u: $ty) -> Self {
                    Number::PosInt(u as u64)
                }
            }
        )*
    };
}

macro_rules! impl_from_signed {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Number {
                #[inline]
                fn from(i:$ty) -> Self {
                    if i < 0 {
                        Number::NegInt(i as i64)
                    }else {
                        Number::PosInt(i as u64)
                    }
                }
            }
        )*
    };
}

macro_rules! impl_from_float {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Number {
                #[inline]
                fn from(f: $ty) -> Self {
                    Number::Float(f as f64)
                }
            }
        )*
    };
}

impl_from_unsigned!(u8, u16, u32, u64, usize);
impl_from_signed!(i8, i16, i32, i64, isize);
impl_from_float!(f32, f64);

pub enum Value {
    Bool(bool),
    Num(Number),
    Map(HashMap<Value, Value>),
    List(Vec<Value>),
    Null,
    String(String),
    Bytes(Vec<u8>),
}
