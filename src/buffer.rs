use crate::geometry::{face::OrientedBlockFace, quad::UnorientedQuad};

pub struct PopBufferGroup<const M: usize, Q: Into<UnorientedQuad>> {
    pub(crate) quads: Vec<Q>,
    pub(crate) buckets: [usize; M],
}

impl<const M: usize, Q: Into<UnorientedQuad>> PopBufferGroup<M, Q> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            quads: Vec::new(),
            buckets: [0; M],
        }
    }
}

pub struct PopBuffer<const M: usize, Q: Into<UnorientedQuad>> {
    pub(crate) groups: [PopBufferGroup<M, Q>; 6],
}

impl<const M: usize, Q: Into<UnorientedQuad>> PopBuffer<M, Q> {
    const EMPTY_GROUP: PopBufferGroup<M, Q> = PopBufferGroup::new();

    #[inline]
    pub const fn new() -> Self {
        Self {
            groups: [Self::EMPTY_GROUP; 6],
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        for group in self.groups.iter_mut() {
            group.quads.clear();
            group.buckets.fill(0);
        }
    }

    #[inline]
    pub fn add_quad(&mut self, face_index: usize, quad: Q, lod: usize) {
        let group = &mut self.groups[face_index];
        group.quads.push(quad);

        for i in 0..lod {
            group.quads.swap(group.buckets[i], group.buckets[i + 1]);
        }

        for i in 0..=lod {
            group.buckets[i] += 1;
        }
    }
}

#[derive(Debug)]
pub struct QuadsBuffer<Q: Into<UnorientedQuad>> {
    pub(crate) groups: [Vec<Q>; 6],
}

impl<Q: Into<UnorientedQuad>> Default for QuadsBuffer<Q> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Q: Into<UnorientedQuad>> QuadsBuffer<Q> {
    const EMPTY_VEC: Vec<Q> = Vec::new();

    #[inline]
    pub const fn new() -> Self {
        Self {
            groups: [Self::EMPTY_VEC; 6],
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        for group in self.groups.iter_mut() {
            group.clear()
        }
    }

    #[inline]
    pub fn num_quads(&self) -> usize {
        let mut quads = 0;
        for group in self.groups.iter() {
            quads += group.len();
        }
        quads
    }

    #[inline]
    pub fn iter_groups(self) -> impl Iterator<Item = (OrientedBlockFace, Vec<Q>)> {
        OrientedBlockFace::FACES.into_iter().zip(self.groups)
    }

    #[inline]
    pub fn iter_quads(self) -> impl Iterator<Item = (OrientedBlockFace, Q)> {
        self.iter_groups()
            .flat_map(|(face, group)| group.into_iter().map(move |quad| (face, quad)))
    }
}
