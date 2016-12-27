use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Neg,
               Index, IndexMut};
use num::Zero;

use alga::general::{ClosedMul, ClosedDiv, ClosedAdd, ClosedSub, ClosedNeg};

use core::{Scalar, Matrix, OwnedMatrix, MatrixSum, MatrixMul, MatrixTrMul};
use core::dimension::Dim;
use core::constraint::{ShapeConstraint, SameNumberOfRows, SameNumberOfColumns, AreMultipliable};
use core::storage::{Storage, StorageMut, OwnedStorage};
use core::allocator::{SameShapeAllocator, Allocator, OwnedAllocator};

/*
 *
 * Indexing.
 *
 */
impl<N: Scalar, R: Dim, C: Dim, S: Storage<N, R, C>> Index<usize> for Matrix<N, R, C, S> {
    type Output = N;

    #[inline]
    fn index(&self, i: usize) -> &N {
        let ij = self.vector_to_matrix_index(i);
        &self[ij]
    }
}


impl<N, R: Dim, C: Dim, S> Index<(usize, usize)> for Matrix<N, R, C, S>
    where N: Scalar,
          S: Storage<N, R, C> {
    type Output = N;

    #[inline]
    fn index(&self, ij: (usize, usize)) -> &N {
        assert!(ij < self.shape(), "Matrix index out of bounds.");
        unsafe { self.get_unchecked(ij.0, ij.1) }
    }
}

// Mutable versions.
impl<N: Scalar, R: Dim, C: Dim, S: StorageMut<N, R, C>> IndexMut<usize> for Matrix<N, R, C, S> {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut N {
        let ij = self.vector_to_matrix_index(i);
        &mut self[ij]
    }
}

impl<N, R: Dim, C: Dim, S> IndexMut<(usize, usize)> for Matrix<N, R, C, S>
    where N: Scalar,
          S: StorageMut<N, R, C> {

    #[inline]
    fn index_mut(&mut self, ij: (usize, usize)) -> &mut N {
        assert!(ij < self.shape(), "Matrix index out of bounds.");
        unsafe { self.get_unchecked_mut(ij.0, ij.1) }
    }
}

/*
 *
 * Neg
 *
 */
impl<N, R: Dim, C: Dim, S> Neg for Matrix<N, R, C, S>
    where N: Scalar + ClosedNeg,
          S: Storage<N, R, C> {
    type Output = OwnedMatrix<N, R, C, S::Alloc>;

    #[inline]
    fn neg(self) -> Self::Output {
        let mut res = self.into_owned();
        res.neg_mut();
        res
    }
}

impl<'a, N, R: Dim, C: Dim, S> Neg for &'a Matrix<N, R, C, S>
    where N: Scalar + ClosedNeg,
          S: Storage<N, R, C> {
    type Output = OwnedMatrix<N, R, C, S::Alloc>;

    #[inline]
    fn neg(self) -> Self::Output {
        -self.clone_owned()
    }
}

impl<N, R: Dim, C: Dim, S> Matrix<N, R, C, S>
    where N: Scalar + ClosedNeg,
          S: StorageMut<N, R, C> {
    /// Negates `self` in-place.
    #[inline]
    pub fn neg_mut(&mut self) {
        for e in self.iter_mut() {
            *e = -*e
        }
    }
}

/*
 *
 * Addition & Substraction
 *
 */
macro_rules! componentwise_binop_impl(
    ($Trait: ident, $method: ident, $bound: ident;
     $TraitAssign: ident, $method_assign: ident) => {
        impl<'b, N, R1, C1, R2, C2, SA, SB> $Trait<&'b Matrix<N, R2, C2, SB>> for Matrix<N, R1, C1, SA>
            where R1: Dim, C1: Dim, R2: Dim, C2: Dim,
                  N: Scalar + $bound,
                  SA: Storage<N, R1, C1>,
                  SB: Storage<N, R2, C2>,
                  SA::Alloc: SameShapeAllocator<N, R1, C1, R2, C2, SA>,
                  ShapeConstraint: SameNumberOfRows<R1, R2> + SameNumberOfColumns<C1, C2> {
            type Output = MatrixSum<N, R1, C1, R2, C2, SA>;

            #[inline]
            fn $method(self, right: &'b Matrix<N, R2, C2, SB>) -> Self::Output {
                assert!(self.shape() == right.shape(), "Matrix addition/subtraction dimensions mismatch.");
                let mut res = self.into_owned_sum::<R2, C2>();

                for (left, right) in res.iter_mut().zip(right.iter()) {
                    *left = left.$method(*right)
                }

                res
            }
        }

        impl<'a, N, R1, C1, R2, C2, SA, SB> $Trait<Matrix<N, R2, C2, SB>> for &'a Matrix<N, R1, C1, SA>
            where R1: Dim, C1: Dim, R2: Dim, C2: Dim,
                  N: Scalar + $bound,
                  SA: Storage<N, R1, C1>,
                  SB: Storage<N, R2, C2>,
                  SB::Alloc: SameShapeAllocator<N, R2, C2, R1, C1, SB>,
                  ShapeConstraint: SameNumberOfRows<R2, R1> + SameNumberOfColumns<C2, C1> {
            type Output = MatrixSum<N, R2, C2, R1, C1, SB>;

            #[inline]
            fn $method(self, right: Matrix<N, R2, C2, SB>) -> Self::Output {
                assert!(self.shape() == right.shape(), "Matrix addition/subtraction dimensions mismatch.");
                let mut res = right.into_owned_sum::<R1, C1>();

                for (left, right) in self.iter().zip(res.iter_mut()) {
                    *right = left.$method(*right)
                }

                res
            }
        }

        impl<N, R1, C1, R2, C2, SA, SB> $Trait<Matrix<N, R2, C2, SB>> for Matrix<N, R1, C1, SA>
            where R1: Dim, C1: Dim, R2: Dim, C2: Dim,
                  N: Scalar + $bound,
                  SA: Storage<N, R1, C1>,
                  SB: Storage<N, R2, C2>,
                  SA::Alloc: SameShapeAllocator<N, R1, C1, R2, C2, SA>,
                  ShapeConstraint: SameNumberOfRows<R1, R2> + SameNumberOfColumns<C1, C2> {
            type Output = MatrixSum<N, R1, C1, R2, C2, SA>;

            #[inline]
            fn $method(self, right: Matrix<N, R2, C2, SB>) -> Self::Output {
                self.$method(&right)
            }
        }

        impl<'a, 'b, N, R1, C1, R2, C2, SA, SB> $Trait<&'b Matrix<N, R2, C2, SB>> for &'a Matrix<N, R1, C1, SA>
            where R1: Dim, C1: Dim, R2: Dim, C2: Dim,
                  N: Scalar + $bound,
                  SA: Storage<N, R1, C1>,
                  SB: Storage<N, R2, C2>,
                  SA::Alloc: SameShapeAllocator<N, R1, C1, R2, C2, SA>,
                  ShapeConstraint: SameNumberOfRows<R1, R2> + SameNumberOfColumns<C1, C2> {
            type Output = MatrixSum<N, R1, C1, R2, C2, SA>;

            #[inline]
            fn $method(self, right: &'b Matrix<N, R2, C2, SB>) -> Self::Output {
                self.clone_owned().$method(right)
            }
        }

        impl<'b, N, R1, C1, R2, C2, SA, SB> $TraitAssign<&'b Matrix<N, R2, C2, SB>> for Matrix<N, R1, C1, SA>
            where R1: Dim, C1: Dim, R2: Dim, C2: Dim,
                  N: Scalar + $bound,
                  SA: StorageMut<N, R1, C1>,
                  SB: Storage<N, R2, C2>,
                  ShapeConstraint: SameNumberOfRows<R1, R2> + SameNumberOfColumns<C1, C2> {

            #[inline]
            fn $method_assign(&mut self, right: &'b Matrix<N, R2, C2, SB>) {
                assert!(self.shape() == right.shape(), "Matrix addition/subtraction dimensions mismatch.");
                for (left, right) in self.iter_mut().zip(right.iter()) {
                    left.$method_assign(*right)
                }
            }
        }

        impl<N, R1, C1, R2, C2, SA, SB> $TraitAssign<Matrix<N, R2, C2, SB>> for Matrix<N, R1, C1, SA>
            where R1: Dim, C1: Dim, R2: Dim, C2: Dim,
                  N: Scalar + $bound,
                  SA: StorageMut<N, R1, C1>,
                  SB: Storage<N, R2, C2>,
                  ShapeConstraint: SameNumberOfRows<R1, R2> + SameNumberOfColumns<C1, C2> {

            #[inline]
            fn $method_assign(&mut self, right: Matrix<N, R2, C2, SB>) {
                self.$method_assign(&right)
            }
        }
    }
);

componentwise_binop_impl!(Add, add, ClosedAdd; AddAssign, add_assign);
componentwise_binop_impl!(Sub, sub, ClosedSub; SubAssign, sub_assign);



/*
 *
 * Multiplication
 *
 */

// Matrix × Scalar
// Matrix / Scalar
macro_rules! componentwise_scalarop_impl(
    ($Trait: ident, $method: ident, $bound: ident;
     $TraitAssign: ident, $method_assign: ident) => {
        impl<N, R: Dim, C: Dim, S> $Trait<N> for Matrix<N, R, C, S>
            where N: Scalar + $bound,
                  S: Storage<N, R, C> {
            type Output = OwnedMatrix<N, R, C, S::Alloc>;

            #[inline]
            fn $method(self, rhs: N) -> Self::Output {
                let mut res = self.into_owned();

                for left in res.iter_mut() {
                    *left = left.$method(rhs)
                }

                res
            }
        }

        impl<'a, N, R: Dim, C: Dim, S> $Trait<N> for &'a Matrix<N, R, C, S>
            where N: Scalar + $bound,
                  S: Storage<N, R, C> {
            type Output = OwnedMatrix<N, R, C, S::Alloc>;

            #[inline]
            fn $method(self, rhs: N) -> Self::Output {
                self.clone_owned().$method(rhs)
            }
        }

        impl<N, R: Dim, C: Dim, S> $TraitAssign<N> for Matrix<N, R, C, S>
            where N: Scalar + $bound,
                  S: StorageMut<N, R, C> {
            #[inline]
            fn $method_assign(&mut self, right: N) {
                for left in self.iter_mut() {
                    left.$method_assign(right)
                }
            }
        }
    }
);

componentwise_scalarop_impl!(Mul, mul, ClosedMul; MulAssign, mul_assign);
componentwise_scalarop_impl!(Div, div, ClosedDiv; DivAssign, div_assign);

macro_rules! left_scalar_mul_impl(
    ($($T: ty),* $(,)*) => {$(
        impl<R: Dim, C: Dim, S> Mul<Matrix<$T, R, C, S>> for $T
            where S: Storage<$T, R, C> {
            type Output = OwnedMatrix<$T, R, C, S::Alloc>;

            #[inline]
            fn mul(self, right: Matrix<$T, R, C, S>) -> Self::Output {
                let mut res = right.into_owned();

                for right in res.iter_mut() {
                    *right = self * *right
                }

                res
            }
        }

        impl<'b, R: Dim, C: Dim, S> Mul<&'b Matrix<$T, R, C, S>> for $T
            where S: Storage<$T, R, C> {
            type Output = OwnedMatrix<$T, R, C, S::Alloc>;

            #[inline]
            fn mul(self, right: &'b Matrix<$T, R, C, S>) -> Self::Output {
                self * right.clone_owned()
            }
        }
    )*}
);

left_scalar_mul_impl!(
    u8, u16, u32, u64, usize,
    i8, i16, i32, i64, isize,
    f32, f64
);



// Matrix × Matrix
impl<'a, 'b, N, R1: Dim, C1: Dim, R2: Dim, C2: Dim, SA, SB> Mul<&'b Matrix<N, R2, C2, SB>>
for &'a Matrix<N, R1, C1, SA>
    where N:  Scalar + Zero + ClosedAdd + ClosedMul,
          SB: Storage<N, R2, C2>,
          SA: Storage<N, R1, C1>,
          SA::Alloc: Allocator<N, R1, C2>,
          ShapeConstraint: AreMultipliable<R1, C1, R2, C2> {
    type Output = MatrixMul<N, R1, C1, C2, SA>;

    #[inline]
    fn mul(self, right: &'b Matrix<N, R2, C2, SB>) -> Self::Output {
        let (nrows1, ncols1) = self.shape();
        let (nrows2, ncols2) = right.shape();

        assert!(ncols1 == nrows2, "Matrix multiplication dimensions mismatch.");

        let mut res: MatrixMul<N, R1, C1, C2, SA> = unsafe {
            Matrix::new_uninitialized_generic(self.data.shape().0, right.data.shape().1)
        };

        for i in 0 .. nrows1 {
            for j in 0 .. ncols2 {
                let mut acc = N::zero();

                unsafe {
                    for k in 0 .. ncols1 {
                        acc = acc + *self.get_unchecked(i, k) * *right.get_unchecked(k, j);
                    }

                    *res.get_unchecked_mut(i, j) = acc;
                }
            }
        }

        res
    }
}

impl<'a, N, R1: Dim, C1: Dim, R2: Dim, C2: Dim, SA, SB> Mul<Matrix<N, R2, C2, SB>>
for &'a Matrix<N, R1, C1, SA>
    where N:  Scalar + Zero + ClosedAdd + ClosedMul,
          SB: Storage<N, R2, C2>,
          SA: Storage<N, R1, C1>,
          SA::Alloc: Allocator<N, R1, C2>,
          ShapeConstraint: AreMultipliable<R1, C1, R2, C2> {
    type Output = MatrixMul<N, R1, C1, C2, SA>;

    #[inline]
    fn mul(self, right: Matrix<N, R2, C2, SB>) -> Self::Output {
        self * &right
    }
}

impl<'b, N, R1: Dim, C1: Dim, R2: Dim, C2: Dim, SA, SB> Mul<&'b Matrix<N, R2, C2, SB>>
for Matrix<N, R1, C1, SA>
    where N:  Scalar + Zero + ClosedAdd + ClosedMul,
          SB: Storage<N, R2, C2>,
          SA: Storage<N, R1, C1>,
          SA::Alloc: Allocator<N, R1, C2>,
          ShapeConstraint: AreMultipliable<R1, C1, R2, C2> {
    type Output = MatrixMul<N, R1, C1, C2, SA>;

    #[inline]
    fn mul(self, right: &'b Matrix<N, R2, C2, SB>) -> Self::Output {
        &self * right
    }
}

impl<N, R1: Dim, C1: Dim, R2: Dim, C2: Dim, SA, SB> Mul<Matrix<N, R2, C2, SB>>
for Matrix<N, R1, C1, SA>
    where N:  Scalar + Zero + ClosedAdd + ClosedMul,
          SB: Storage<N, R2, C2>,
          SA: Storage<N, R1, C1>,
          SA::Alloc: Allocator<N, R1, C2>,
          ShapeConstraint: AreMultipliable<R1, C1, R2, C2> {
    type Output = MatrixMul<N, R1, C1, C2, SA>;

    #[inline]
    fn mul(self, right: Matrix<N, R2, C2, SB>) -> Self::Output {
        &self * &right
    }
}

// FIXME: this is too restrictive:
//    − we can't use `a *= b` when `a` is a mutable slice.
//    − we can't use `a *= b` when C2 is not equal to C1.
impl<N, R1, C1, R2, SA, SB> MulAssign<Matrix<N, R2, C1, SB>> for Matrix<N, R1, C1, SA>
    where R1: Dim, C1: Dim, R2: Dim,
          N:  Scalar + Zero + ClosedAdd + ClosedMul,
          SB: Storage<N, R2, C1>,
          SA: OwnedStorage<N, R1, C1>,
          ShapeConstraint: AreMultipliable<R1, C1, R2, C1>,
          SA::Alloc: OwnedAllocator<N, R1, C1, SA> {
    #[inline]
    fn mul_assign(&mut self, right: Matrix<N, R2, C1, SB>) {
        *self = &*self * right
    }
}

impl<'b, N, R1, C1, R2, SA, SB> MulAssign<&'b Matrix<N, R2, C1, SB>> for Matrix<N, R1, C1, SA>
    where R1: Dim, C1: Dim, R2: Dim,
          N:  Scalar + Zero + ClosedAdd + ClosedMul,
          SB: Storage<N, R2, C1>,
          SA: OwnedStorage<N, R1, C1>,
          ShapeConstraint: AreMultipliable<R1, C1, R2, C1>,
          // FIXME: this is too restrictive. See comments for the non-ref version.
          SA::Alloc: OwnedAllocator<N, R1, C1, SA> {
    #[inline]
    fn mul_assign(&mut self, right: &'b Matrix<N, R2, C1, SB>) {
        *self = &*self * right
    }
}


impl<N, R1: Dim, C1: Dim, SA> Matrix<N, R1, C1, SA>
    where N:  Scalar + Zero + ClosedAdd + ClosedMul,
          SA: Storage<N, R1, C1> {
    /// Equivalent to `self.transpose() * right`.
    #[inline]
    pub fn tr_mul<R2: Dim, C2: Dim, SB>(&self, right: &Matrix<N, R2, C2, SB>) -> MatrixTrMul<N, R1, C1, C2, SA>
        where SB: Storage<N, R2, C2>,
              SA::Alloc: Allocator<N, C1, C2>,
              ShapeConstraint: AreMultipliable<C1, R1, R2, C2> {
        let (nrows1, ncols1) = self.shape();
        let (nrows2, ncols2) = right.shape();

        assert!(nrows1 == nrows2, "Matrix multiplication dimensions mismatch.");

        let mut res: MatrixTrMul<N, R1, C1, C2, SA> = unsafe {
            Matrix::new_uninitialized_generic(self.data.shape().1, right.data.shape().1)
        };

        for i in 0 .. ncols1 {
            for j in 0 .. ncols2 {
                let mut acc = N::zero();

                unsafe {
                    for k in 0 .. nrows1 {
                        acc = acc + *self.get_unchecked(k, i) * *right.get_unchecked(k, j);
                    }

                    *res.get_unchecked_mut(i, j) = acc;
                }
            }
        }

        res
    }
}