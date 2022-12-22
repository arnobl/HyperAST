use std::{fmt::Debug, marker::PhantomData};

use num_traits::{cast, one, PrimInt};

use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, Initializable, PostOrder,
    PostOrderIterable,
};
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{optimal::zs::ZsMatcher, similarity_metrics};
use hyper_ast::types::{LabelStore, NodeStore, SlicedLabel, Tree, WithHashs};

use super::bottom_up_matcher::BottomUpMatcher;
use crate::decompressed_tree_store::SimpleZsTree as ZsTree;

/// TODO wait for #![feature(adt_const_params)] #95174 to be improved
///
/// it will allow to make use complex types as const generics
/// ie. make the different threshold neater
pub struct GreedyBottomUpMatcher<
    'a,
    Dsrc,
    Ddst,
    T: 'a + Tree + WithHashs,
    S,
    LS: LabelStore<SlicedLabel, I = T::Label>,
    M: MonoMappingStore,
    const SIZE_THRESHOLD: usize = 1000,
    const SIM_THRESHOLD_NUM: u64 = 1,
    const SIM_THRESHOLD_DEN: u64 = 2,
> {
    label_store: &'a LS,
    internal: BottomUpMatcher<'a, Dsrc, Ddst, T, S, M>,
}

impl<
        'a,
        Dsrc: 'a
            + DecompressedTreeStore<'a, T, M::Src>
            + DecompressedWithParent<'a, T, M::Src>
            + PostOrder<'a, T, M::Src>,
        Ddst: 'a
            + DecompressedTreeStore<'a, T, M::Dst>
            + DecompressedWithParent<'a, T, M::Dst>
            + PostOrder<'a, T, M::Dst>,
        // IdD: PrimInt + std::ops::SubAssign + Debug,
        T: Tree + WithHashs,
        S,
        LS: LabelStore<SlicedLabel, I = T::Label>,
        M: MonoMappingStore,
        const SIZE_THRESHOLD: usize,  // = 1000,
        const SIM_THRESHOLD_NUM: u64, // = 1,
        const SIM_THRESHOLD_DEN: u64, // = 2,
    > Into<BottomUpMatcher<'a, Dsrc, Ddst, T, S, M>>
    for GreedyBottomUpMatcher<
        'a,
        Dsrc,
        Ddst,
        T,
        S,
        LS,
        M,
        SIZE_THRESHOLD,
        SIM_THRESHOLD_NUM,
        SIM_THRESHOLD_DEN,
    >
{
    fn into(self) -> BottomUpMatcher<'a, Dsrc, Ddst, T, S, M> {
        self.internal
    }
}

/// TODO PostOrder might not be necessary
impl<
        'a,
        Dsrc: 'a
            + DecompressedTreeStore<'a, T, M::Src>
            + DecompressedWithParent<'a, T, M::Src>
            + PostOrder<'a, T, M::Src>
            + PostOrderIterable<'a, T, M::Src>
            + Initializable<'a, T>
            + ContiguousDescendants<'a, T, M::Src>,
        Ddst: 'a
            + DecompressedTreeStore<'a, T, M::Dst>
            + DecompressedWithParent<'a, T, M::Dst>
            + PostOrder<'a, T, M::Dst>
            + PostOrderIterable<'a, T, M::Dst>
            + Initializable<'a, T>
            + ContiguousDescendants<'a, T, M::Dst>,
        T: Tree + WithHashs,
        S: 'a + NodeStore<T::TreeId, R<'a> = T>,
        LS: 'a + LabelStore<SlicedLabel, I = T::Label>,
        M: MonoMappingStore,
        const SIZE_THRESHOLD: usize,
        const SIM_THRESHOLD_NUM: u64,
        const SIM_THRESHOLD_DEN: u64,
    >
    GreedyBottomUpMatcher<
        'a,
        Dsrc,
        Ddst,
        T,
        S,
        LS,
        M,
        SIZE_THRESHOLD,
        SIM_THRESHOLD_NUM,
        SIM_THRESHOLD_DEN,
    >
where
    T::TreeId: 'a + Clone + Debug,
    T::Type: Debug,
    M::Src: 'a + PrimInt + std::ops::SubAssign + Debug,
    M::Dst: 'a + PrimInt + std::ops::SubAssign + Debug,
{
    pub fn new(
        node_store: &'a S,
        label_store: &'a LS,
        src_arena: Dsrc,
        dst_arena: Ddst,
        mappings: M,
    ) -> Self {
        Self {
            label_store,
            internal: BottomUpMatcher {
                node_store,
                src_arena,
                dst_arena,
                mappings,
                phantom: PhantomData,
            },
        }
    }

    pub fn matchh(
        compressed_node_store: &'a S,
        label_store: &'a LS,
        src: &'a T::TreeId,
        dst: &'a T::TreeId,
        mappings: M,
    ) -> Self {
        let mut matcher = Self::new(
            compressed_node_store,
            label_store,
            Dsrc::new(compressed_node_store, src),
            Ddst::new(compressed_node_store, dst),
            mappings,
        );
        matcher.internal.mappings.topit(
            matcher.internal.src_arena.len() + 1,
            matcher.internal.dst_arena.len() + 1,
        );
        Self::execute(&mut matcher);
        matcher
    }

    pub fn execute(&mut self) {
        assert_eq!(
            // TODO move it inside the arena ...
            self.internal.src_arena.root(),
            cast::<_, M::Src>(self.internal.src_arena.len()).unwrap() - one()
        );
        assert!(self.internal.src_arena.len() > 0);
        // println!("mappings={}", self.internal.mappings.len());
        // // WARN it is in postorder and it depends on decomp store
        // // -1 as root is handled after forloop
        for a in self.internal.src_arena.iter_df_post() {
            // let a: IdD = cast(i).unwrap(); // might be problematic as it depends on decompressed store?
            if self.internal.src_arena.parent(&a).is_none() {
                continue;
            }
            if !(self.internal.mappings.is_src(&a) || !self.src_has_children(a)) {
                let candidates = self.internal.get_dst_candidates(&a);
                let mut best = None;
                let mut max: f64 = -1.;
                for cand in candidates {
                    let sim = similarity_metrics::SimilarityMeasure::range(
                        &self.internal.src_arena.descendants_range(&a),
                        &self.internal.dst_arena.descendants_range(&cand),
                        &self.internal.mappings,
                    )
                    .dice();
                    if sim > max && sim >= SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64 {
                        max = sim;
                        best = Some(cand);
                    }
                }

                if let Some(best) = best {
                    self.last_chance_match_zs::<M>(a, best);
                    self.internal.mappings.link(a, best);
                }
            }
        }
        // for root
        self.internal.mappings.link(
            self.internal.src_arena.root(),
            self.internal.dst_arena.root(),
        );
        self.last_chance_match_zs::<M>(
            self.internal.src_arena.root(),
            self.internal.dst_arena.root(),
        );
        // println!("nodes:{}", c);
        // println!("nodes:{}", c2);
    }

    fn src_has_children(&mut self, src: M::Src) -> bool {
        let r = self
            .internal
            .node_store
            .resolve(&self.internal.src_arena.original(&src))
            .has_children();
        assert_eq!(r, self.internal.src_arena.lld(&src) <= src);
        r
    }

    pub(crate) fn last_chance_match_zs<MZs>(&mut self, src: M::Src, dst: M::Dst)
    where
        MZs: MonoMappingStore,
        MZs::Src: 'a + PrimInt + Debug + std::ops::SubAssign,
        MZs::Dst: 'a + PrimInt + Debug + std::ops::SubAssign,
    {
        let src_offset = self.internal.src_arena.first_descendant(&src);
        let dst_offset = self.internal.dst_arena.first_descendant(&dst);
        let src_s = self
            .internal
            .src_arena
            .descendants_count(self.internal.node_store, &src);
        let dst_s = self
            .internal
            .dst_arena
            .descendants_count(self.internal.node_store, &dst);
        let o_src = self.internal.src_arena.original(&src);
        let o_dst = self.internal.dst_arena.original(&dst);
        if src_s < cast(SIZE_THRESHOLD).unwrap() || dst_s < cast(SIZE_THRESHOLD).unwrap() {
            let matcher: ZsMatcher<MZs, ZsTree<T, MZs::Src>, ZsTree<T, MZs::Dst>> = {
                let node_store = self.internal.node_store;
                let label_store = self.label_store;
                let src_arena = ZsTree::new(node_store, &o_src);
                let dst_arena = ZsTree::new(node_store, &o_dst);
                ZsMatcher::match_with(node_store, label_store, src_arena, dst_arena)
            };
            let mappings = matcher.mappings;
            for (i, t) in mappings.iter() {
                //remapping
                let src: M::Src = src_offset + cast(i - num_traits::one()).unwrap();
                let dst: M::Dst = dst_offset + cast(t - num_traits::one()).unwrap();
                // use it
                let bbb =
                    !self.internal.mappings.is_src(&src) && !self.internal.mappings.is_dst(&dst);
                let tsrc = self
                    .internal
                    .node_store
                    .resolve(&self.internal.src_arena.original(&src))
                    .get_type();
                let tdst = self
                    .internal
                    .node_store
                    // .resolve(&matcher.src_arena.tree(&t))
                    .resolve(&self.internal.dst_arena.original(&dst))
                    .get_type();
                if bbb {
                    if tsrc == tdst {
                        self.internal.mappings.link(src, dst);
                    }
                }
            }
        }
    }
}
