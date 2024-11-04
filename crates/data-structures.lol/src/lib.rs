#![feature(
    linked_list_cursors,
    maybe_uninit_uninit_array,
    maybe_uninit_array_assume_init,
    maybe_uninit_slice,
    maybe_uninit_write_slice,
    const_swap,
    slice_swap_unchecked,
    const_replace,
    array_chunks,
    const_maybe_uninit_assume_init,
    const_maybe_uninit_uninit_array,
    const_maybe_uninit_write,
    const_trait_impl,
    const_option_ext,
    const_for,
    portable_simd,
    async_iterator,
    extend_one,
    array_windows,
    const_box,
    iterator_try_collect,
    hash_raw_entry,
    box_into_inner,
    never_type,
    cell_update,
    let_chains
)]
#![allow(unexpected_cfgs)]
#![cfg_attr(feature = "generic_const_exprs", feature(generic_const_exprs))]
#![allow(incomplete_features)]

pub mod atomic_cell;
pub mod atomic_slot;
pub mod circular_buffer;
pub mod cursor_vec;
pub mod file_tree;
pub mod index_map;
pub mod linked_set;
pub mod max_only_heap;
pub mod maybe_owned;
pub mod maybe_owned_mut;
pub mod non_empty_vec;
pub mod ordmap;
#[cfg(feature = "prefetch-stream")]
pub mod prefetch_stream;
pub mod ref_set;
pub mod reusable_bufs;
#[cfg(feature = "generic_const_exprs")]
pub mod ring_buffer;
pub mod shared;
#[cfg(feature = "pool")]
pub mod shared_slab;
pub mod slice_deque;
// #[cfg(feature = "small-str")]
// pub mod small_str;
pub mod stack;
pub mod stack_string;
pub mod static_or_boxed;
// pub mod str;
pub mod string_dst;
pub mod take_once;
pub mod tree;
pub mod visitor;
// pub mod concurrent;

/// Identical to MaybeUninit::uninit_array, but usable on stable.
const fn uninit_array<const N: usize, T>() -> [std::mem::MaybeUninit<T>; N] {
    // SAFETY: an array of uninitialized elements doesn't need initialization.
    // the std docs for MaybeUninit use this exact method as an example.
    unsafe { std::mem::MaybeUninit::uninit().assume_init() }
}

macro_rules! export_loom_std_modules {
    ($($path:ident),* $(,)?) => {
        $(
            pub(crate) mod $path {
                #[cfg(loom)]
                pub use loom::$path::*;
                #[cfg(not(loom))]
                pub use std::$path::*;
            }
        )*
    };
}

export_loom_std_modules!(cell, hint, thread, sync);
