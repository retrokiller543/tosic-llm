// tosic_llm/src/utils.rs

pub mod doc;

use std::vec::IntoIter;

pub enum SingleOrMultiple<T> {
    Single(T),
    Multiple(Vec<T>),
}

impl<T> SingleOrMultiple<T> {
    pub fn from_iter(iter: impl IntoIterator<Item = T>) -> Self {
        Self::Multiple(iter.into_iter().collect())
    }
}

impl<T> From<T> for SingleOrMultiple<T> {
    fn from(item: T) -> Self {
        Self::Single(item)
    }
}

impl<T> From<Vec<T>> for SingleOrMultiple<T> {
    fn from(items: Vec<T>) -> Self {
        Self::Multiple(items)
    }
}

impl<T> IntoIterator for SingleOrMultiple<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Single(item) => vec![item].into_iter(),
            Self::Multiple(items) => items.into_iter(),
        }
    }
}
