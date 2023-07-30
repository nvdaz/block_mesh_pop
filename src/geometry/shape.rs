use bevy_math::UVec3;

use super::face::OrientedBlockFace;

pub struct ChunkShape<const X: u32, const Y: u32, const Z: u32>;

impl<const X: u32, const Y: u32, const Z: u32> ChunkShape<X, Y, Z> {
    pub const STRIDES: UVec3 = UVec3::new(1, X, X * Y);

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
    pub fn inner_iter() -> InnerChunkShapeIterator<X, Y, Z> {
        InnerChunkShapeIterator::<X, Y, Z>::new()
    }

    #[inline]
    pub fn slice_iter(face: OrientedBlockFace, n: u32) -> InnerChunkShapeSliceIterator<X, Y, Z> {
        InnerChunkShapeSliceIterator::<X, Y, Z>::new(face, n)
    }
}

pub struct InnerChunkShapeIterator<const X: u32, const Y: u32, const Z: u32> {
    x: u32,
    y: u32,
    z: u32,
}

impl<const X: u32, const Y: u32, const Z: u32> Default for InnerChunkShapeIterator<X, Y, Z> {
    fn default() -> Self {
        Self { x: 0, y: 1, z: 1 }
    }
}

impl<const X: u32, const Y: u32, const Z: u32> InnerChunkShapeIterator<X, Y, Z> {
    fn new() -> Self {
        Self::default()
    }
}

impl<const X: u32, const Y: u32, const Z: u32> Iterator for InnerChunkShapeIterator<X, Y, Z> {
    type Item = UVec3;

    fn next(&mut self) -> Option<Self::Item> {
        if self.x < X - 2 {
            if self.x == 0 {
                self.x = 1;
            } else {
                self.x += 1;
            }
            Some(UVec3::new(self.x, self.y, self.z))
        } else {
            self.x = 1;

            if self.y < Y - 2 {
                self.y += 1;
                Some(UVec3::new(self.x, self.y, self.z))
            } else {
                self.y = 1;

                if self.z < Z - 2 {
                    self.z += 1;
                    Some(UVec3::new(self.x, self.y, self.z))
                } else {
                    None
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
