use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

struct GeneratorInner {
    ids: Vec<usize>,
    allocated: usize,
}

pub struct Generator {
    inner: Arc<Mutex<GeneratorInner>>,
    chunk_size: usize,
}

impl Generator {
    pub fn generate(&mut self) -> Id {
        if let Ok(mut inner) = self.inner.lock() {
            if let Some(value) = inner.ids.pop() {
                Id {
                    inner: Arc::new(IdInner {
                        value,
                        parent: Arc::clone(&self.inner),
                    }),
                }
            } else {
                let old_allocated = inner.allocated;
                inner.allocated += self.chunk_size;

                let last_index = inner.allocated - 1;

                for value in old_allocated..last_index {
                    inner.ids.push(value);
                }

                Id {
                    inner: Arc::new(IdInner {
                        value: last_index,
                        parent: Arc::clone(&self.inner),
                    }),
                }
            }
        } else {
            panic!("Could not generate new id!");
        }
    }
}

pub struct GeneratorBuilder {
    chunk_size: usize,
    default_size: usize,
}

impl Default for GeneratorBuilder {
    fn default() -> Self {
        Self {
            chunk_size: 128,
            default_size: 128,
        }
    }
}

impl GeneratorBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        assert!(chunk_size > 0);
        self.chunk_size = chunk_size;
        self
    }

    pub fn with_size(mut self, size: usize) -> Self {
        self.default_size = size;
        self
    }

    pub fn build(self) -> Generator {
        Generator {
            chunk_size: self.chunk_size,
            inner: Arc::new(Mutex::new(GeneratorInner {
                ids: (0..self.default_size).collect(),
                allocated: self.default_size,
            })),
        }
    }
}

struct IdInner {
    value: usize,
    parent: Arc<Mutex<GeneratorInner>>,
}

impl Drop for IdInner {
    fn drop(&mut self) {
        if let Ok(mut parent) = self.parent.lock() {
            parent.ids.push(self.value);
        }
    }
}

impl Eq for IdInner {}
impl PartialEq for IdInner {
    fn eq(&self, rhs: &Self) -> bool {
        self.value.eq(&rhs.value)
    }
}

impl Debug for IdInner {
    fn fmt(
        &self,
        format: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        format.write_str(&format!("Id({})", self.value))
    }
}

impl Hash for IdInner {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Id {
    inner: Arc<IdInner>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn allocate_ids_no_duplicates() {
        let mut already_allocated = HashSet::<usize>::new();

        let size = 1000;

        let mut generator = GeneratorBuilder::new().with_size(size).build();

        let mut references = Vec::with_capacity(size);

        for _ in 0..size {
            let val = generator.generate();
            assert!(already_allocated.get(&val.inner.value).is_some() == false);
            already_allocated.insert(val.inner.value);
            references.push(val);
        }
    }

    #[test]
    fn returning_ids_will_allow_them_to_be_reallocated() {
        let mut already_allocated = HashSet::<usize>::new();

        let size = 1000;

        let mut generator = GeneratorBuilder::new().with_size(size).build();

        let mut references = Vec::with_capacity(size);

        for _ in 0..size {
            let val = generator.generate();
            assert!(already_allocated.get(&val.inner.value).is_some() == false);
            already_allocated.insert(val.inner.value);
            references.push(val);
        }

        drop(references);

        let mut references = Vec::with_capacity(size);

        for _ in 0..size {
            let val = generator.generate();
            assert!(already_allocated.get(&val.inner.value).is_some());
            references.push(val);
        }
    }

    #[test]
    fn dont_return_ownership_if_live_reference() {
        let mut already_allocated = HashSet::<usize>::new();

        let size = 1000;

        let mut generator = GeneratorBuilder::new().with_size(size).build();

        let mut references = Vec::with_capacity(size);

        for _ in 0..size {
            let val = generator.generate();
            assert!(already_allocated.get(&val.inner.value).is_some() == false);
            already_allocated.insert(val.inner.value);
            references.push(val);
        }

        let more_references = references.clone();

        drop(references);

        let mut references = Vec::with_capacity(size);

        for _ in 0..size {
            let val = generator.generate();
            assert!(already_allocated.get(&val.inner.value).is_some() == false);
            already_allocated.insert(val.inner.value);
            references.push(val);
        }

        drop(more_references);
        drop(references);

        let mut references = Vec::with_capacity(size);

        for _ in 0..size {
            let val = generator.generate();
            assert!(already_allocated.get(&val.inner.value).is_some());
            references.push(val);
        }
    }
}
