use crate::{
    diag::Diag,
    linop::{BiLinOp, BiPrecond, LinOp, Precond},
    ComplexField, Conjugate, MatMut, MatRef, Parallelism,
};
use dyn_stack::{PodStack, SizeOverflow, StackReq};

impl<E: ComplexField, ViewE: Conjugate<Canonical = E>> LinOp<E> for Diag<ViewE> {
    #[inline]
    fn apply_req(
        &self,
        rhs_ncols: usize,
        parallelism: Parallelism,
    ) -> Result<StackReq, SizeOverflow> {
        self.as_ref().apply_req(rhs_ncols, parallelism)
    }

    #[inline]
    fn nrows(&self) -> usize {
        self.as_ref().nrows()
    }

    #[inline]
    fn ncols(&self) -> usize {
        self.as_ref().ncols()
    }

    fn apply(
        &self,
        out: MatMut<'_, E>,
        rhs: MatRef<'_, E>,
        parallelism: Parallelism,
        stack: &mut PodStack,
    ) {
        self.as_ref().apply(out, rhs, parallelism, stack)
    }

    fn conj_apply(
        &self,
        out: MatMut<'_, E>,
        rhs: MatRef<'_, E>,
        parallelism: Parallelism,
        stack: &mut PodStack,
    ) {
        self.as_ref().conj_apply(out, rhs, parallelism, stack)
    }
}

impl<E: ComplexField, ViewE: Conjugate<Canonical = E>> BiLinOp<E> for Diag<ViewE> {
    #[inline]
    fn transpose_apply_req(
        &self,
        rhs_ncols: usize,
        parallelism: Parallelism,
    ) -> Result<StackReq, SizeOverflow> {
        self.as_ref().transpose_apply_req(rhs_ncols, parallelism)
    }

    fn transpose_apply(
        &self,
        out: MatMut<'_, E>,
        rhs: MatRef<'_, E>,
        parallelism: Parallelism,
        stack: &mut PodStack,
    ) {
        self.as_ref().transpose_apply(out, rhs, parallelism, stack)
    }

    fn adjoint_apply(
        &self,
        out: MatMut<'_, E>,
        rhs: MatRef<'_, E>,
        parallelism: Parallelism,
        stack: &mut PodStack,
    ) {
        self.as_ref().adjoint_apply(out, rhs, parallelism, stack)
    }
}

impl<E: ComplexField, ViewE: Conjugate<Canonical = E>> Precond<E> for Diag<ViewE> {
    fn apply_in_place_req(
        &self,
        rhs_ncols: usize,
        parallelism: Parallelism,
    ) -> Result<StackReq, SizeOverflow> {
        self.as_ref().apply_in_place_req(rhs_ncols, parallelism)
    }

    fn apply_in_place(&self, rhs: MatMut<'_, E>, parallelism: Parallelism, stack: &mut PodStack) {
        self.as_ref().apply_in_place(rhs, parallelism, stack)
    }

    fn conj_apply_in_place(
        &self,
        rhs: MatMut<'_, E>,
        parallelism: Parallelism,
        stack: &mut PodStack,
    ) {
        self.as_ref().conj_apply_in_place(rhs, parallelism, stack)
    }
}
impl<E: ComplexField, ViewE: Conjugate<Canonical = E>> BiPrecond<E> for Diag<ViewE> {
    fn transpose_apply_in_place_req(
        &self,
        rhs_ncols: usize,
        parallelism: Parallelism,
    ) -> Result<StackReq, SizeOverflow> {
        self.as_ref()
            .transpose_apply_in_place_req(rhs_ncols, parallelism)
    }

    fn transpose_apply_in_place(
        &self,
        rhs: MatMut<'_, E>,
        parallelism: Parallelism,
        stack: &mut PodStack,
    ) {
        self.as_ref()
            .transpose_apply_in_place(rhs, parallelism, stack)
    }

    fn adjoint_apply_in_place(
        &self,
        rhs: MatMut<'_, E>,
        parallelism: Parallelism,
        stack: &mut PodStack,
    ) {
        self.as_ref()
            .adjoint_apply_in_place(rhs, parallelism, stack)
    }
}
