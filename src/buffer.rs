use crate::geometry::{face::OrientedBlockFace, quad::UnorientedQuad};

pub struct PopBuffer<const M: usize, Q: Into<UnorientedQuad>> {
    pub(crate) groups: [QuadBuffer<Q>; M],
}

impl<const M: usize, Q: Into<UnorientedQuad> + Clone> PopBuffer<M, Q> {
    const EMPTY_QUAD_BUFFER: QuadBuffer<Q> = QuadBuffer::new();
    #[inline]
    pub const fn new() -> Self {
        Self {
            groups: [Self::EMPTY_QUAD_BUFFER; M],
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        for group in self.groups.iter_mut() {
            group.reset();
        }
    }

    #[inline]
    pub fn add_quad(&mut self, face_index: usize, quad: Q, lod: usize) {
        let group = &mut self.groups[lod];

        group.groups[face_index].push(quad);
    }

    #[inline]
    pub fn num_quads(&self) -> usize {
        let mut quads = 0;
        for group in self.groups.iter() {
            quads += group.num_quads();
        }
        quads
    }

    #[inline]
    pub fn num_quads_lod(&self, lod: usize) -> usize {
        let mut quads = 0;
        for group in self.groups.iter().rev().take(M - lod) {
            quads += group.num_quads();
        }
        quads
    }

    #[inline]
    pub fn get_buckets(&self) -> [u32; 8] {
        let mut buckets = [0; 8];

        for (max_lod, group) in self.groups.iter().enumerate() {
            for lod in 0..=max_lod {
                buckets[lod] += group.num_quads() as u32;
            }
        }

        buckets
    }

    #[inline]
    pub fn iter_quads(self) -> impl Iterator<Item = (OrientedBlockFace, Q)> {
        self.groups
            .into_iter()
            .rev()
            .flat_map(|group| group.iter_quads())
    }

    #[inline]
    pub fn iter_quads_lod(self, lod: usize) -> impl Iterator<Item = (OrientedBlockFace, Q)> {
        self.groups
            .into_iter()
            .rev()
            .take(M - lod)
            .flat_map(|group| group.iter_quads())
    }

    // #[inline]
    // pub fn separate(self) -> (QuadBuffer<Q>, [[usize; M]; 6]) {
    //     let mut quads: [Vec<Q>; 6] = [Self::EMPTY_VEC; 6];
    //     let mut buckets = [[0; M]; 6];

    //     for (index, mut group) in self.groups.into_iter().enumerate() {
    //         mem::swap(&mut quads[index], &mut group.quads);
    //         mem::swap(&mut buckets[index], &mut group.buckets);
    //     }

    //     let quad_buffer = QuadBuffer { groups: quads };

    //     (quad_buffer, buckets)
    // }

    // #[inline]
    // pub fn separate_flat(self) -> (Vec<Q>, [usize; M]) {
    //     let (quad_buffer, buckets) = self.separate();

    //     let mut quads: Vec<Q> = Vec::new();
    //     let mut flat_buckets = [0; M];

    //     for lod in 0..M {
    //         for face in 0..6 {
    //             let start_index = if lod == 0 { 0 } else { buckets[face][lod - 1] };
    //             let end_index = buckets[face][lod];
    //             for i in start_index..end_index {
    //                 let quad = quad_buffer.groups[face][i].clone();
    //                 quads.push(quad);
    //             }
    //             flat_buckets[lod] += end_index - start_index;
    //         }
    //     }

    //     (quads, flat_buckets)
    // }
}

#[derive(Debug)]
pub struct QuadBuffer<Q: Into<UnorientedQuad>> {
    pub(crate) groups: [Vec<Q>; 6],
}

impl<Q: Into<UnorientedQuad>> Default for QuadBuffer<Q> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Q: Into<UnorientedQuad>> QuadBuffer<Q> {
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

// impl<const M: usize, Q: Into<UnorientedQuad>> From<PopBuffer<M, Q>> for QuadBuffer<Q> {
//     #[inline]
//     fn from(value: PopBuffer<M, Q>) -> Self {
//         Self {
//             groups: value.groups.map(|group| group.quads),
//         }
//     }
// }

#[derive(Debug)]
pub struct VisitedBuffer {
    // This really should be a smaller type (u8)
    // but a u32 vec is 2x faster for some reason.
    pub(crate) visited: Vec<u8>,
}

impl VisitedBuffer {
    #[inline]
    pub fn new(size: usize) -> Self {
        Self {
            visited: vec![0; size],
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.visited.fill(0);
    }
}
