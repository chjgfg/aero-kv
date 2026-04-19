use crate::{storage::storage::Log, error::Result};
use std::{collections::btree_map, iter::Iterator};

pub struct DiskIterator<'a> {
    pub inner: btree_map::Range<'a, Vec<u8>, (u64, u32)>,
    pub log: &'a mut Log,
}

impl<'a> DiskIterator<'a> {
    fn map(&mut self, item: (&Vec<u8>, &(u64, u32))) -> <Self as Iterator>::Item {
        let (key, (offset, value_size)) = item;
        let value = self.log.read_value(*offset, *value_size)?;
        Ok((key.clone(), value))
    }
}

impl<'a> Iterator for DiskIterator<'a> {
    type Item = Result<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| self.map(item)) // 正确使用 next 按顺序迭代
    }
}

impl<'a> DoubleEndedIterator for DiskIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|item| self.map(item)) // 正确使用 next_back 按逆序迭代
    }
}
