use crate::*;
use binread::BinReaderExt;
use bv::BitVec;
use std::borrow::Cow;
use std::mem::size_of;

pub struct LumpReader<R> {
    inner: R,
    length: usize,
}

impl<'a> LumpReader<Cursor<Cow<'a, [u8]>>> {
    pub fn new(data: Cow<'a, [u8]>) -> Self {
        let length = data.len();
        let reader = Cursor::new(data);
        LumpReader {
            inner: reader,
            length,
        }
    }
}

impl<R: BinReaderExt + Read> LumpReader<R> {
    pub fn read_entities(&mut self) -> BspResult<Entities> {
        let mut entities = String::with_capacity(self.length);
        self.inner.read_to_string(&mut entities)?;
        Ok(Entities { entities })
    }

    pub fn read_vec<F, T>(&mut self, mut f: F) -> BspResult<Vec<T>>
    where
        F: FnMut(&mut LumpReader<R>) -> BspResult<T>,
    {
        let num_entries = self.length / size_of::<T>();
        let mut entries = Vec::with_capacity(num_entries);

        for _ in 0..num_entries {
            entries.push(f(self)?);
        }

        Ok(entries)
    }

    pub fn read<T: BinRead>(&mut self) -> BspResult<T> {
        Ok(self.inner.read_le()?)
    }

    pub fn read_visdata(&mut self) -> BspResult<VisData> {
        if (self.length as usize) < std::mem::size_of::<u32>() * 2 {
            return Ok(VisData::default());
        }

        let n_vecs = self.inner.read_le()?;
        let sz_vecs = self.inner.read_le()?;
        let vecs_size = n_vecs as u64 * sz_vecs as u64;
        let mut vecs = Vec::with_capacity(
            vecs_size
                .try_into()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        );
        self.inner
            .by_ref()
            .take(vecs_size as u64)
            .read_to_end(&mut vecs)?;

        if (vecs.len() as u64) < vecs_size {
            return Err(BspError::UnexpectedEOF);
        }

        if (vecs.len() as u64) > vecs_size {
            return Err(BspError::UnexpectedExtraData);
        }

        let vecs = BitVec::from_bits(vecs);

        let vis_data = VisData {
            n_vecs,
            sz_vecs,
            vecs,
        };
        Ok(vis_data)
    }
}
