use bevy_math::UVec3;

use super::face::{FaceStrides, OrientedBlockFace};

pub struct ChunkShape<const X: u32, const Y: u32, const Z: u32>;

impl<const X: u32, const Y: u32, const Z: u32> ChunkShape<X, Y, Z> {
    pub const STRIDES: UVec3 = UVec3::new(1, X, X * Y);
    pub const SHAPE: UVec3 = UVec3::new(X, Y, Z);

    pub const FACE_STRIDES: [FaceStrides; 6] = [
        Self::face_strides(OrientedBlockFace::FACES[0]),
        Self::face_strides(OrientedBlockFace::FACES[1]),
        Self::face_strides(OrientedBlockFace::FACES[2]),
        Self::face_strides(OrientedBlockFace::FACES[3]),
        Self::face_strides(OrientedBlockFace::FACES[4]),
        Self::face_strides(OrientedBlockFace::FACES[5]),
    ];

    #[inline]
    pub const fn linearize(position: UVec3) -> u32 {
        position.x
            + Self::STRIDES.y.wrapping_mul(position.y)
            + Self::STRIDES.z.wrapping_mul(position.z)
    }

    #[inline]
    pub const fn delinearize(mut index: u32) -> UVec3 {
        let z = index / Self::STRIDES.z;
        index -= z * Self::STRIDES.z;
        let y = index / Self::STRIDES.y;
        let x = index % Self::STRIDES.y;
        UVec3::new(x, y, z)
    }

    #[inline]
    pub const fn face_strides(face: OrientedBlockFace) -> FaceStrides {
        let n = Self::linearize(face.n);
        let n = if face.is_front {
            n
        } else {
            0u32.wrapping_sub(n)
        };

        FaceStrides {
            n,
            u: Self::linearize(face.u),
            v: Self::linearize(face.v),
        }
    }

    #[inline]
    pub fn localize(face: OrientedBlockFace, n: u32, u: u32, v: u32) -> UVec3 {
        let n = if !face.is_front {
            face.n.dot(Self::SHAPE) - n - 1
        } else {
            n
        };

        face.n * n + face.u * u + face.v * v
    }

    #[inline]
    pub fn inner_iter<const F: usize>() -> InnerChunkShapeIterator<X, Y, Z, F> {
        InnerChunkShapeIterator::<X, Y, Z, F>::new()
    }

    #[inline]
    pub fn slice_iter(face: OrientedBlockFace, n: u32) -> InnerChunkShapeSliceIterator<X, Y, Z> {
        InnerChunkShapeSliceIterator::<X, Y, Z>::new(face, n)
    }
}

pub struct InnerChunkShapeIterator<const X: u32, const Y: u32, const Z: u32, const F: usize> {
    position: UVec3,
    max_n: u32,
    max_u: u32,
    max_v: u32,
}

impl<const X: u32, const Y: u32, const Z: u32, const F: usize> InnerChunkShapeIterator<X, Y, Z, F> {
    const FACE: OrientedBlockFace = OrientedBlockFace::FACES[F];

    fn new() -> Self {
        let n = if Self::FACE.is_front {
            Self::FACE.n.dot(ChunkShape::<X, Y, Z>::SHAPE) - 2
        } else {
            1
        };

        // Omit u axis since it will be added in the first iteration and `position` represents the previous value.
        let position = Self::FACE.n * n + Self::FACE.v;

        let max_n = Self::FACE.n.dot(ChunkShape::<X, Y, Z>::SHAPE);
        let max_u = Self::FACE.u.dot(ChunkShape::<X, Y, Z>::SHAPE);
        let max_v = Self::FACE.v.dot(ChunkShape::<X, Y, Z>::SHAPE);

        Self {
            position,
            max_n,
            max_u,
            max_v,
        }
    }
}

impl<const X: u32, const Y: u32, const Z: u32, const F: usize> Iterator
    for InnerChunkShapeIterator<X, Y, Z, F>
{
    type Item = UVec3;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if Self::FACE.u.dot(self.position) < self.max_u - 2 {
            self.position += Self::FACE.u;
            Some(self.position)
        } else {
            self.position -= Self::FACE.u * (Self::FACE.u.dot(self.position) - 1);

            if Self::FACE.v.dot(self.position) < self.max_v - 2 {
                self.position += Self::FACE.v;
                Some(self.position)
            } else {
                self.position -= Self::FACE.v * (Self::FACE.v.dot(self.position) - 1);

                match (Self::FACE.is_front, Self::FACE.n.dot(self.position)) {
                    (true, n) if n > 1 => {
                        self.position -= Self::FACE.n;
                        Some(self.position)
                    }
                    (false, n) if n < self.max_n - 2 => {
                        self.position += Self::FACE.n;
                        Some(self.position)
                    }
                    _ => None,
                }
            }
        }
    }
}

pub struct InnerChunkShapeSliceIterator<const X: u32, const Y: u32, const Z: u32> {
    face: OrientedBlockFace,
    n: u32,
    u: u32,
    v: u32,
    max_u: u32,
    max_v: u32,
}

impl<const X: u32, const Y: u32, const Z: u32> InnerChunkShapeSliceIterator<X, Y, Z> {
    fn new(face: OrientedBlockFace, n: u32) -> Self {
        let shape = UVec3::new(X, Y, Z);
        Self {
            face,
            n,
            u: 0,
            v: 1,
            max_u: face.u.dot(shape),
            max_v: face.v.dot(shape),
        }
    }
}

impl<const X: u32, const Y: u32, const Z: u32> Iterator for InnerChunkShapeSliceIterator<X, Y, Z> {
    type Item = UVec3;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.u < self.max_u - 2 {
            self.u += 1;
            Some(self.face.n * self.n + self.face.u * self.u + self.face.v * self.v)
        } else {
            self.u = 1;

            if self.v < self.max_v - 2 {
                self.v += 1;
                Some(self.face.n * self.n + self.face.u * self.u + self.face.v * self.v)
            } else {
                None
            }
        }
    }
}
