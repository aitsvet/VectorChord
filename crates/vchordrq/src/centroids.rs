// This software is licensed under a dual license model:
//
// GNU Affero General Public License v3 (AGPLv3): You may use, modify, and
// distribute this software under the terms of the AGPLv3.
//
// Elastic License v2 (ELv2): You may also use, modify, and distribute this
// software under the Elastic License v2, which has specific restrictions.
//
// We welcome any commercial collaboration or support. For inquiries
// regarding the licenses, please contact us at:
// vectorchord-inquiry@tensorchord.ai
//
// Copyright (c) 2025 TensorChord Inc.

use crate::Opaque;
use crate::closure_lifetime_binder::{id_0, id_1};
use crate::operator::*;
use crate::tape::by_next;
use crate::tuples::*;
use index::accessor::{Accessor1, FunctionalAccessor};
use index::relation::{Page, RelationRead};
use vector::VectorOwned;

pub fn read<
    'a,
    R: RelationRead + 'a,
    O: Operator,
    A: Accessor1<<O::Vector as Vector>::Element, <O::Vector as Vector>::Metadata>,
>(
    mut prefetch: impl Iterator<Item = R::ReadGuard<'a>>,
    head: u16,
    accessor: A,
) -> A::Output {
    let mut cursor = Err(head);
    let mut result = accessor;
    while let Err(head) = cursor {
        let guard = prefetch.next().expect("data corruption");
        let bytes = guard.get(head).expect("data corruption");
        let tuple = CentroidTuple::<O::Vector>::deserialize_ref(bytes);
        result.push(tuple.elements());
        cursor = tuple.metadata_or_head();
    }
    if prefetch.next().is_some() {
        panic!("data corruption");
    }
    result.finish(cursor.expect("data corruption"))
}

#[derive(Debug, Clone)]
pub struct VectorCollector<V: Vector> {
    dim: u32,
    elements: Vec<V::Element>,
}

impl<V: Vector> VectorCollector<V> {
    pub fn new(dim: u32) -> Self {
        Self {
            dim,
            elements: Vec::new(),
        }
    }
}

impl<V: Vector> Accessor1<V::Element, V::Metadata> for VectorCollector<V> {
    type Output = V;

    #[inline(always)]
    fn push(&mut self, input: &[V::Element]) {
        self.elements.extend(input);
    }

    #[inline(always)]
    fn finish(self, metadata: V::Metadata) -> Self::Output {
        V::pack(self.dim, self.elements, metadata)
    }
}

pub struct CentroidInfo<V: VectorOwned> {
    pub level: u32,
    pub id: u32,
    pub vector: V,
}

pub fn list_all<R: RelationRead, O: Operator>(index: &R) -> Vec<CentroidInfo<O::Vector>>
where
    R::Page: Page<Opaque = Opaque>,
{
    let meta_guard = index.read(0);
    let meta_bytes = meta_guard.get(1).expect("data corruption");
    let meta_tuple = MetaTuple::deserialize_ref(meta_bytes);
    let dim = meta_tuple.dim();
    let height_of_root = meta_tuple.height_of_root();

    let mut results = Vec::new();

    let root_prefetch = meta_tuple.centroid_prefetch().to_vec();
    let root_head = meta_tuple.centroid_head();
    let root_first = meta_tuple.first();

    drop(meta_guard);

    let root_vector = read::<R, O, _>(
        root_prefetch.iter().map(|&id| index.read(id)),
        root_head,
        VectorCollector::<O::Vector>::new(dim),
    );

    results.push(CentroidInfo {
        level: height_of_root - 1,
        id: 0,
        vector: root_vector,
    });

    type State = Vec<(u32, Vec<u32>, u16)>;
    let mut state: State = vec![(root_first, root_prefetch, root_head)];

    for level in (1..height_of_root).rev() {
        let mut next_state: State = Vec::new();
        let mut centroid_id = 0u32;

        for (first, _parent_prefetch, _parent_head) in state {
            crate::tape::read_h1_tape::<R, _, _>(
                by_next(index, first),
                || FunctionalAccessor::new((), id_0(|_, _| ()), id_1(|_, _| [(); _])),
                |(), head, _norm, child_first, prefetch| {
                    let prefetch_vec = prefetch.to_vec();
                    let centroid_vector = read::<R, O, _>(
                        prefetch_vec.iter().map(|&id| index.read(id)),
                        head,
                        VectorCollector::<O::Vector>::new(dim),
                    );

                    results.push(CentroidInfo {
                        level: level - 1,
                        id: centroid_id,
                        vector: centroid_vector,
                    });
                    centroid_id += 1;

                    if level > 1 {
                        next_state.push((child_first, prefetch_vec, head));
                    }
                },
            );
        }

        state = next_state;
    }

    results
}
