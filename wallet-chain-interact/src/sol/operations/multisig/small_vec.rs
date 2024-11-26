use borsh::{BorshDeserialize, BorshSerialize};
use std::io::{Read, Write};
use std::marker::PhantomData;

#[derive(Clone, Debug, Default)]
pub struct SmallVec<L, T>(Vec<T>, PhantomData<L>);

impl<L, T> SmallVec<L, T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<L, T> From<SmallVec<L, T>> for Vec<T> {
    fn from(val: SmallVec<L, T>) -> Self {
        val.0
    }
}

impl<L, T> From<Vec<T>> for SmallVec<L, T> {
    fn from(val: Vec<T>) -> Self {
        Self(val, PhantomData)
    }
}

impl<T: BorshSerialize> BorshSerialize for SmallVec<u8, T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let len = u8::try_from(self.len()).map_err(|_| std::io::ErrorKind::InvalidInput)?;
        // Write the length of the vector as u8.
        writer.write_all(&len.to_le_bytes())?;

        // Write the vector elements.
        for item in &self.0 {
            item.serialize(writer)?;
        }
        Ok(())
    }
}

impl<T: BorshSerialize> BorshSerialize for SmallVec<u16, T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let len = u16::try_from(self.len()).map_err(|_| std::io::ErrorKind::InvalidInput)?;
        // Write the length of the vector as u16.
        writer.write_all(&len.to_le_bytes())?;

        // Write the vector elements.
        for item in &self.0 {
            item.serialize(writer)?;
        }
        Ok(())
    }
}

impl<L, T> BorshDeserialize for SmallVec<L, T>
where
    L: BorshDeserialize + Into<u32>,
    T: BorshDeserialize,
{
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let len: u32 = L::deserialize_reader(reader)?.into();

        let vec = if len == 0 {
            Vec::new()
        } else {
            let mut result = Vec::with_capacity(len as usize);
            for _ in 0..len {
                result.push(T::deserialize_reader(reader)?);
            }
            result
        };

        Ok(SmallVec(vec, PhantomData))
    }
}
