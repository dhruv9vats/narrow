use super::{Array, ArrayType};
use crate::{
    buffer::{BufferType, VecBuffer},
    validity::Validity,
};

/// Struct array types.
pub trait StructArrayType: ArrayType {
    /// The array type that stores items of this struct. Note this differs from the `ArrayType` array because that wraps this array
    type Array<Buffer: BufferType>;
}

pub struct StructArray<
    T: StructArrayType,
    const NULLABLE: bool = false,
    BitmapBuffer: BufferType = VecBuffer,
>(<<T as StructArrayType>::Array<BitmapBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer>)
where
    <T as StructArrayType>::Array<BitmapBuffer>: Validity<NULLABLE>;

impl<T: StructArrayType, const NULLABLE: bool, BitmapBuffer: BufferType> Array
    for StructArray<T, NULLABLE, BitmapBuffer>
where
    <T as StructArrayType>::Array<BitmapBuffer>: Validity<NULLABLE>,
{
}

impl<T: StructArrayType, U, const NULLABLE: bool, BitmapBuffer: BufferType> FromIterator<U>
    for StructArray<T, NULLABLE, BitmapBuffer>
where
    <T as StructArrayType>::Array<BitmapBuffer>: Validity<NULLABLE>,
    <<T as StructArrayType>::Array<BitmapBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer>:
        FromIterator<U>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter() {
        // Definition
        // #[derive(Array)]
        struct Foo<'a> {
            a: u32,
            b: Option<()>,
            c: (),
            d: Option<[u128; 2]>,
            e: bool,
            f: &'a [u8],
            g: String,
        }
        // These impls below can all be generated.
        impl<'a> ArrayType for Foo<'a> {
            type Array<Buffer: BufferType> = StructArray<Foo<'a>, false, Buffer>;
        }
        struct FooArray<'a, Buffer: BufferType> {
            a: <u32 as ArrayType>::Array<Buffer>,
            b: <Option<()> as ArrayType>::Array<Buffer>,
            c: <() as ArrayType>::Array<Buffer>,
            d: <Option<[u128; 2]> as ArrayType>::Array<Buffer>,
            e: <bool as ArrayType>::Array<Buffer>,
            f: <&'a [u8] as ArrayType>::Array<Buffer>,
            g: <String as ArrayType>::Array<Buffer>,
        }
        impl<'a, Buffer: BufferType> FromIterator<Foo<'a>> for FooArray<'a, Buffer>
        where
            <u32 as ArrayType>::Array<Buffer>: Default + Extend<u32>,
            <Option<()> as ArrayType>::Array<Buffer>: Default + Extend<Option<()>>,
            <() as ArrayType>::Array<Buffer>: Default + Extend<()>,
            <Option<[u128; 2]> as ArrayType>::Array<Buffer>: Default + Extend<Option<[u128; 2]>>,
            <bool as ArrayType>::Array<Buffer>: Default + Extend<bool>,
            <&'a [u8] as ArrayType>::Array<Buffer>: Default + Extend<&'a [u8]>,
            <String as ArrayType>::Array<Buffer>: Default + Extend<String>,
        {
            fn from_iter<T: IntoIterator<Item = Foo<'a>>>(iter: T) -> Self {
                let (a, (b, (c, (d, (e, (f, g)))))) = iter
                    .into_iter()
                    .map(
                        |Foo {
                             a,
                             b,
                             c,
                             d,
                             e,
                             f,
                             g,
                         }| (a, (b, (c, (d, (e, (f, g)))))),
                    )
                    .unzip();
                Self {
                    a,
                    b,
                    c,
                    d,
                    e,
                    f,
                    g,
                }
            }
        }
        impl<'a> StructArrayType for Foo<'a> {
            type Array<Buffer: BufferType> = FooArray<'a, Buffer>;
        }

        // And then:
        let input = [
            Foo {
                a: 1,
                b: None,
                c: (),
                d: Some([1, 2]),
                e: false,
                f: &[1],
                g: "a".to_string(),
            },
            Foo {
                a: 2,
                b: Some(()),
                c: (),
                d: Some([3, 4]),
                e: true,
                f: &[2, 3],
                g: "s".to_string(),
            },
            Foo {
                a: 3,
                b: None,
                c: (),
                d: None,
                e: true,
                f: &[4],
                g: "d".to_string(),
            },
            Foo {
                a: 4,
                b: None,
                c: (),
                d: None,
                e: true,
                f: &[],
                g: "f".to_string(),
            },
        ];
        let array = input.into_iter().collect::<StructArray<Foo>>();
        assert_eq!(array.0.a.into_iter().collect::<Vec<_>>(), &[1, 2, 3, 4]);
        assert_eq!(
            array.0.b.into_iter().collect::<Vec<_>>(),
            &[None, Some(()), None, None]
        );
        assert_eq!(array.0.c.into_iter().collect::<Vec<_>>(), &[(), (), (), ()]);
        assert_eq!(
            array.0.d.into_iter().collect::<Vec<_>>(),
            &[Some([1, 2]), Some([3, 4]), None, None]
        );
        assert_eq!(
            array.0.e.into_iter().collect::<Vec<_>>(),
            &[false, true, true, true]
        );
        assert_eq!(
            array.0.f.0.data.into_iter().collect::<Vec<_>>(),
            &[1, 2, 3, 4]
        );
        assert_eq!(
            array.0.f.0.offsets.into_iter().collect::<Vec<_>>(),
            &[0, 1, 3, 4, 4]
        );
        assert_eq!(
            array.0.g.0 .0 .0.data.into_iter().collect::<Vec<_>>(),
            &[97, 115, 100, 102] // a s d f
        );
        assert_eq!(
            array.0.g.0 .0 .0.offsets.into_iter().collect::<Vec<_>>(),
            &[0, 1, 2, 3, 4]
        );
    }
}
