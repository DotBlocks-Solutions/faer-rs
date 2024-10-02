use matref::{partition_by_le, partition_by_lt};

use super::*;
use crate::assert;

/// Sparse matrix view in column-major format, either compressed or uncompressed.
pub struct SparseColMatMut<'a, I: Index, E: Entity, R: Shape = usize, C: Shape = usize> {
    pub(crate) symbolic: SymbolicSparseColMatRef<'a, I, R, C>,
    pub(crate) values: SliceGroupMut<'a, E>,
}

impl<'short, I: Index, E: Entity, R: Shape, C: Shape> Reborrow<'short>
    for SparseColMatMut<'_, I, E, R, C>
{
    type Target = SparseColMatRef<'short, I, E, R, C>;

    #[inline]
    fn rb(&'short self) -> Self::Target {
        SparseColMatRef {
            symbolic: self.symbolic,
            values: self.values.rb(),
        }
    }
}

impl<'short, I: Index, E: Entity, R: Shape, C: Shape> ReborrowMut<'short>
    for SparseColMatMut<'_, I, E, R, C>
{
    type Target = SparseColMatMut<'short, I, E, R, C>;

    #[inline]
    fn rb_mut(&'short mut self) -> Self::Target {
        SparseColMatMut {
            symbolic: self.symbolic,
            values: self.values.rb_mut(),
        }
    }
}

impl<'a, I: Index, E: Entity, R: Shape, C: Shape> IntoConst for SparseColMatMut<'a, I, E, R, C> {
    type Target = SparseColMatRef<'a, I, E, R, C>;

    #[inline]
    fn into_const(self) -> Self::Target {
        SparseColMatRef {
            symbolic: self.symbolic,
            values: self.values.into_const(),
        }
    }
}

impl<'a, I: Index, E: Entity, R: Shape, C: Shape> SparseColMatMut<'a, I, E, R, C> {
    /// Creates a new sparse matrix view.
    ///
    /// # Panics
    ///
    /// Panics if the length of `values` is not equal to the length of
    /// `symbolic.row_indices()`.
    #[inline]
    #[track_caller]
    pub fn new(symbolic: SymbolicSparseColMatRef<'a, I, R, C>, values: SliceMut<'a, E>) -> Self {
        let values = SliceGroupMut::new(values);
        assert!(symbolic.row_indices().len() == values.len());
        Self { symbolic, values }
    }

    /// Returns the number of rows of the matrix.
    #[inline]
    pub fn nrows(&self) -> R {
        self.symbolic.nrows
    }
    /// Returns the number of columns of the matrix.
    #[inline]
    pub fn ncols(&self) -> C {
        self.symbolic.ncols
    }

    /// Returns the number of rows and columns of the matrix.
    #[inline]
    pub fn shape(&self) -> (R, C) {
        (self.nrows(), self.ncols())
    }

    /// Returns a view over `self`.
    #[inline]
    pub fn as_ref(&self) -> SparseColMatRef<'_, I, E, R, C> {
        (*self).rb()
    }

    /// Returns a mutable view over `self`.
    ///
    /// Note that the symbolic structure cannot be changed through this view.
    #[inline]
    pub fn as_mut(&mut self) -> SparseColMatMut<'_, I, E, R, C> {
        (*self).rb_mut()
    }

    /// Copies `self` into a newly allocated matrix.
    ///
    /// # Note
    /// Allows unsorted matrices, producing an unsorted output.
    #[inline]
    pub fn to_owned(&self) -> Result<SparseColMat<I, E::Canonical, R, C>, FaerError>
    where
        E: Conjugate<Canonical: ComplexField>,
    {
        self.rb().to_owned()
    }

    /// Copies `self` into a newly allocated matrix with sorted indices.
    ///
    /// # Note
    /// Allows unsorted matrices, producing a sorted output.
    #[inline]
    pub fn to_sorted(&self) -> Result<SparseColMat<I, E::Canonical, R, C>, FaerError>
    where
        E: Conjugate<Canonical: ComplexField>,
    {
        self.as_ref().to_sorted()
    }

    /// Copies `self` into a newly allocated dense matrix
    #[inline]
    pub fn to_dense(&self) -> Mat<E::Canonical, R, C>
    where
        E: Conjugate<Canonical: ComplexField>,
    {
        self.as_ref().to_dense()
    }

    /// Copies `self` into a newly allocated matrix, with row-major order.
    ///
    /// # Note
    /// Allows unsorted matrices, producing a sorted output.
    #[inline]
    pub fn to_row_major(&self) -> Result<SparseRowMat<I, E::Canonical, R, C>, FaerError>
    where
        E: Conjugate,
        E::Canonical: ComplexField,
    {
        self.rb().to_row_major()
    }

    /// Returns a view over the transpose of `self` in row-major format.
    #[inline]
    pub fn transpose(self) -> SparseRowMatRef<'a, I, E, C, R> {
        self.into_const().transpose()
    }

    /// Returns a view over the transpose of `self` in row-major format.
    #[inline]
    pub fn transpose_mut(self) -> SparseRowMatMut<'a, I, E, C, R> {
        SparseRowMatMut {
            symbolic: SymbolicSparseRowMatRef {
                nrows: self.symbolic.ncols,
                ncols: self.symbolic.nrows,
                row_ptr: self.symbolic.col_ptr,
                row_nnz: self.symbolic.col_nnz,
                col_ind: self.symbolic.row_ind,
            },
            values: self.values,
        }
    }

    /// Returns a view over the conjugate of `self`.
    #[inline]
    pub fn conjugate(self) -> SparseColMatRef<'a, I, E::Conj, R, C>
    where
        E: Conjugate,
    {
        self.into_const().conjugate()
    }

    /// Returns a view over the conjugate of `self`.
    #[inline]
    pub fn conjugate_mut(self) -> SparseColMatMut<'a, I, E::Conj, R, C>
    where
        E: Conjugate,
    {
        SparseColMatMut {
            symbolic: self.symbolic,
            values: unsafe {
                SliceGroupMut::<'a, E::Conj>::new(transmute_unchecked::<
                    GroupFor<E, &mut [UnitFor<E::Conj>]>,
                    GroupFor<E::Conj, &mut [UnitFor<E::Conj>]>,
                >(E::faer_map(
                    self.values.into_inner(),
                    |slice| {
                        let len = slice.len();
                        core::slice::from_raw_parts_mut(
                            slice.as_ptr() as *mut UnitFor<E> as *mut UnitFor<E::Conj>,
                            len,
                        )
                    },
                )))
            },
        }
    }

    /// Returns a view over the canonical view of `self`, along with whether it has been conjugated
    /// or not.
    #[inline]
    pub fn canonicalize(self) -> (SparseColMatRef<'a, I, E::Canonical, R, C>, Conj)
    where
        E: Conjugate,
    {
        self.into_const().canonicalize()
    }

    /// Returns a view over the canonical view of `self`, along with whether it has been conjugated
    /// or not.
    #[inline]
    pub fn canonicalize_mut(self) -> (SparseColMatMut<'a, I, E::Canonical, R, C>, Conj)
    where
        E: Conjugate,
    {
        (
            SparseColMatMut {
                symbolic: self.symbolic,
                values: unsafe {
                    SliceGroupMut::<'a, E::Canonical>::new(transmute_unchecked::<
                        GroupFor<E, &mut [UnitFor<E::Canonical>]>,
                        GroupFor<E::Canonical, &mut [UnitFor<E::Canonical>]>,
                    >(E::faer_map(
                        self.values.into_inner(),
                        |slice| {
                            let len = slice.len();
                            core::slice::from_raw_parts_mut(
                                slice.as_mut_ptr() as *mut UnitFor<E> as *mut UnitFor<E::Canonical>,
                                len,
                            )
                        },
                    )))
                },
            },
            if coe::is_same::<E, E::Canonical>() {
                Conj::No
            } else {
                Conj::Yes
            },
        )
    }

    /// Returns a view over the conjugate transpose of `self`.
    #[inline]
    pub fn adjoint(self) -> SparseRowMatRef<'a, I, E::Conj, C, R>
    where
        E: Conjugate,
    {
        self.into_const().adjoint()
    }

    /// Returns a view over the conjugate transpose of `self`.
    #[inline]
    pub fn adjoint_mut(self) -> SparseRowMatMut<'a, I, E::Conj, C, R>
    where
        E: Conjugate,
    {
        self.transpose_mut().conjugate_mut()
    }

    /// Returns the numerical values of the matrix.
    #[inline]
    pub fn values(self) -> Slice<'a, E> {
        self.into_const().values()
    }

    /// Returns the numerical values of the matrix.
    #[inline]
    pub fn values_mut(self) -> SliceMut<'a, E> {
        self.values.into_inner()
    }

    /// Returns the numerical values of column `j` of the matrix.
    ///
    /// # Panics:
    ///
    /// Panics if `j >= ncols`.
    #[inline]
    #[track_caller]
    pub fn values_of_col(self, j: Idx<C>) -> Slice<'a, E> {
        self.into_const().values_of_col(j)
    }

    /// Returns the numerical values of column `j` of the matrix.
    ///
    /// # Panics:
    ///
    /// Panics if `j >= ncols`.
    #[inline]
    #[track_caller]
    pub fn values_of_col_mut(self, j: Idx<C>) -> SliceMut<'a, E> {
        let range = self.symbolic().col_range(j);
        self.values.subslice(range).into_inner()
    }

    /// Returns the symbolic structure of the matrix.
    #[inline]
    pub fn symbolic(&self) -> SymbolicSparseColMatRef<'a, I, R, C> {
        self.symbolic
    }

    /// Decomposes the matrix into the symbolic part and the numerical values.
    #[inline]
    pub fn parts(self) -> (SymbolicSparseColMatRef<'a, I, R, C>, Slice<'a, E>) {
        self.into_const().parts()
    }

    /// Decomposes the matrix into the symbolic part and the numerical values.
    #[inline]
    pub fn parts_mut(self) -> (SymbolicSparseColMatRef<'a, I, R, C>, SliceMut<'a, E>) {
        (self.symbolic, self.values.into_inner())
    }

    /// Returns the number of symbolic non-zeros in the matrix.
    ///
    /// The value is guaranteed to be less than `I::Signed::MAX`.
    ///
    /// # Note
    /// Allows unsorted matrices, but the output is a count of all the entries, including the
    /// duplicate ones.
    #[inline]
    pub fn compute_nnz(&self) -> usize {
        self.symbolic.compute_nnz()
    }

    /// Returns the column pointers.
    #[inline]
    pub fn col_ptrs(&self) -> &'a [I] {
        self.symbolic.col_ptrs()
    }

    /// Returns the count of non-zeros per column of the matrix.
    #[inline]
    pub fn nnz_per_col(&self) -> Option<&'a [I]> {
        self.symbolic.col_nnz
    }

    /// Returns the row indices.
    #[inline]
    pub fn row_indices(&self) -> &'a [I] {
        self.symbolic.row_ind
    }

    /// Returns the row indices of column `j`.
    ///
    /// # Panics
    ///
    /// Panics if `j >= self.ncols()`.
    #[inline]
    #[track_caller]
    pub fn row_indices_of_col_raw(&self, j: Idx<C>) -> &'a [Idx<R, I>] {
        self.symbolic.row_indices_of_col_raw(j)
    }

    /// Returns the row indices of column `j`.
    ///
    /// # Panics
    ///
    /// Panics if `j >= self.ncols()`.
    #[inline]
    #[track_caller]
    pub fn row_indices_of_col(
        &self,
        j: Idx<C>,
    ) -> impl 'a + ExactSizeIterator + DoubleEndedIterator<Item = Idx<R>> {
        self.symbolic.row_indices_of_col(j)
    }

    /// Returns the range that the column `j` occupies in `self.row_indices()`.
    ///
    /// # Panics
    ///
    /// Panics if `j >= self.ncols()`.
    #[inline]
    #[track_caller]
    pub fn col_range(&self, j: Idx<C>) -> Range<usize> {
        self.symbolic.col_range(j)
    }

    /// Returns the range that the column `j` occupies in `self.row_indices()`.
    ///
    /// # Safety
    ///
    /// The behavior is undefined if `j >= self.ncols()`.
    #[inline]
    #[track_caller]
    pub unsafe fn col_range_unchecked(&self, j: Idx<C>) -> Range<usize> {
        self.symbolic.col_range_unchecked(j)
    }

    /// Returns a reference to the value at the given index, or None if the symbolic structure
    /// doesn't contain it, or contains multiple indices with the given index.
    ///
    /// # Panics
    /// Panics if `row >= self.nrows()`  
    /// Panics if `col >= self.ncols()`  
    #[track_caller]
    pub fn get(self, row: Idx<R>, col: Idx<C>) -> Option<Ref<'a, E>> {
        self.into_const().get(row, col)
    }

    /// Returns a reference to the value at the given index, or None if the symbolic structure
    /// doesn't contain it, or contains multiple values with the given index.
    ///
    /// # Panics
    /// Panics if `row >= self.nrows()`.  
    /// Panics if `col >= self.ncols()`.  
    #[track_caller]
    pub fn get_mut(self, row: Idx<R>, col: Idx<C>) -> Option<Mut<'a, E>> {
        let values = self.get_all_mut(row, col);
        if E::faer_first(E::faer_as_ref(&values)).len() == 1 {
            Some(E::faer_map(values, |slice| &mut slice[0]))
        } else {
            None
        }
    }

    /// Returns a reference to a slice containing the values at the given index using a binary
    /// search.
    ///
    /// # Panics
    /// Panics if `row >= self.nrows()`.  
    /// Panics if `col >= self.ncols()`.  
    #[track_caller]
    pub fn get_all(self, row: Idx<R>, col: Idx<C>) -> Slice<'a, E> {
        self.into_const().get_all(row, col)
    }

    /// Returns a reference to a slice containing the values at the given index using a binary
    /// search.
    ///
    /// # Panics
    /// Panics if `row >= self.nrows()`.  
    /// Panics if `col >= self.ncols()`.  
    #[track_caller]
    pub fn get_all_mut(self, row: Idx<R>, col: Idx<C>) -> SliceMut<'a, E> {
        assert!(row < self.nrows());
        assert!(col < self.ncols());

        let row = I::truncate(row.unbound());
        let start = self
            .symbolic()
            .row_indices_of_col_raw_unbound(col)
            .partition_point(partition_by_lt(row));
        let end = start
            + self.symbolic().row_indices_of_col_raw_unbound(col)[start..]
                .partition_point(partition_by_le(row));

        E::faer_map(self.values_of_col_mut(col), |slice| &mut slice[start..end])
    }

    /// Fill the matrix from a previously created value order.
    /// The provided values must correspond to the same indices that were provided in the
    /// function call from which the order was created.
    ///
    /// # Note
    /// The symbolic structure is not changed.
    #[track_caller]
    #[inline]
    pub fn fill_from_order_and_values(
        &mut self,
        order: &ValuesOrder<I>,
        values: Slice<'_, E>,
        mode: FillMode,
    ) where
        E: ComplexField,
    {
        imp(
            self.values.rb_mut(),
            order,
            SliceGroup::<'_, E>::new(values),
            mode,
        );

        fn imp<I: Index, E: ComplexField>(
            mut old_values: SliceGroupMut<'_, E>,
            order: &ValuesOrder<I>,
            new_values: SliceGroup<'_, E>,
            mode: FillMode,
        ) {
            {
                let nnz = order.argsort.len();
                assert!(new_values.len() == nnz);
                assert!(order.nnz == old_values.len());
            }
            let all_nnz = order.all_nnz;
            let mut dst = old_values.rb_mut();

            let mut pos = 0usize;
            let mut pos_unique = usize::MAX;
            let mut current_bit = TOP_BIT;

            match mode {
                FillMode::Replace => {
                    while pos < all_nnz {
                        let argsort_pos = order.argsort[pos];
                        let extracted_bit = argsort_pos & TOP_BIT;
                        let argsort_pos = argsort_pos & TOP_BIT_MASK;

                        let val = new_values.read(argsort_pos);
                        if extracted_bit != current_bit {
                            pos_unique = pos_unique.wrapping_add(1);
                            dst.write(pos_unique, val);
                        } else {
                            let old_val = dst.read(pos_unique);
                            dst.write(pos_unique, old_val.faer_add(val));
                        }

                        current_bit = extracted_bit;

                        pos += 1;
                    }
                }
                FillMode::Add => {
                    while pos < all_nnz {
                        let argsort_pos = order.argsort[pos];
                        let extracted_bit = argsort_pos & TOP_BIT;
                        let argsort_pos = argsort_pos & TOP_BIT_MASK;

                        let val = new_values.read(argsort_pos);
                        if extracted_bit != current_bit {
                            pos_unique = pos_unique.wrapping_add(1);
                        }

                        let old_val = dst.read(pos_unique);
                        dst.write(pos_unique, old_val.faer_add(val));

                        current_bit = extracted_bit;

                        pos += 1;
                    }
                }
            }
        }
    }

    /// Returns the input matrix with the given shape after checking that it matches the
    /// current shape.
    #[inline]
    pub fn as_shape<V: Shape, H: Shape>(
        self,
        nrows: V,
        ncols: H,
    ) -> SparseColMatRef<'a, I, E, V, H> {
        SparseColMatRef {
            symbolic: self.symbolic.as_shape(nrows, ncols),
            values: self.values.into_const(),
        }
    }

    /// Returns the input matrix with dynamic shape.
    #[inline]
    pub fn as_dyn(self) -> SparseColMatRef<'a, I, E> {
        SparseColMatRef {
            symbolic: self.symbolic.as_dyn(),
            values: self.values.into_const(),
        }
    }

    /// Returns the input matrix with the given shape after checking that it matches the
    /// current shape.
    #[inline]
    pub fn as_shape_mut<V: Shape, H: Shape>(
        self,
        nrows: V,
        ncols: H,
    ) -> SparseColMatMut<'a, I, E, V, H> {
        SparseColMatMut {
            symbolic: self.symbolic.as_shape(nrows, ncols),
            values: self.values,
        }
    }

    /// Returns the input matrix with dynamic shape.
    #[inline]
    pub fn as_dyn_mut(self) -> SparseColMatMut<'a, I, E> {
        SparseColMatMut {
            symbolic: self.symbolic.as_dyn(),
            values: self.values,
        }
    }
}

impl<I: Index, E: Entity> core::fmt::Debug for SparseColMatMut<'_, I, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.as_ref().fmt(f)
    }
}
