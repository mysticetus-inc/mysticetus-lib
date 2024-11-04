use std::collections::BTreeMap;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use data_structures::ordmap::OrdMap;
use rand::Rng;

pub struct CharIter {
    range: std::ops::Range<char>,
}

impl Iterator for CharIter {
    type Item = (char, u32);

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|ch| (ch, ch as u32))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

pub fn char_iter() -> CharIter {
    CharIter {
        range: '\0'..char::MAX,
    }
}

pub fn btree_map_extend(c: &mut Criterion) {
    c.bench_function("btree_map_extend", |b| {
        b.iter_batched(
            || (char_iter(), BTreeMap::new()),
            |(iter, mut map)| map.extend(black_box(iter)),
            criterion::BatchSize::SmallInput,
        )
    });
}

pub fn ordmap_extend(c: &mut Criterion) {
    c.bench_function("ordmap_extend", |b| {
        b.iter_batched(
            || {
                let iter = char_iter();
                let (low, high) = iter.size_hint();
                (iter, OrdMap::with_capacity(high.unwrap_or(low)))
            },
            |(iter, mut map)| map.extend(black_box(iter)),
            criterion::BatchSize::SmallInput,
        )
    });
}

pub fn btree_map_insert(c: &mut Criterion) {
    let mut map = BTreeMap::new();
    let mut rng = rand::thread_rng();

    c.bench_function("btree_map_insert", |b| {
        b.iter_batched(
            || rng.gen::<char>(),
            |ch| black_box(map.insert(black_box(ch), black_box(ch as u32))),
            criterion::BatchSize::SmallInput,
        )
    });
}

pub fn ordmap_insert(c: &mut Criterion) {
    let mut map = OrdMap::new();
    let mut rng = rand::thread_rng();

    c.bench_function("ordmap_insert", |b| {
        b.iter_batched(
            || rng.gen::<char>(),
            |ch| black_box(map.insert(black_box(ch), black_box(ch as u32))),
            criterion::BatchSize::SmallInput,
        )
    });
}

use rand::seq::IteratorRandom;

fn rand_chars_vec(count: usize) -> Vec<(char, u32)> {
    let mut rng = rand::thread_rng();
    char_iter().choose_multiple(&mut rng, count)
}

const COUNT: usize = 250;

pub fn ordmap_lookup(c: &mut Criterion) {
    let map = rand_chars_vec(COUNT)
        .into_iter()
        .collect::<OrdMap<char, u32>>();

    let mut rng = rand::thread_rng();

    c.bench_function("ordmap_lookup", move |b| {
        b.iter_batched(
            || rng.gen::<char>(),
            |ch| black_box(map.contains_key(black_box(&ch))),
            criterion::BatchSize::SmallInput,
        )
    });
}

pub fn btree_map_lookup(c: &mut Criterion) {
    let map = rand_chars_vec(COUNT)
        .into_iter()
        .collect::<BTreeMap<char, u32>>();

    let mut rng = rand::thread_rng();

    c.bench_function("btree_map_lookup", move |b| {
        b.iter_batched(
            || rng.gen::<char>(),
            |ch| black_box(map.contains_key(black_box(&ch))),
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    btree_map_insert,
    // btree_map_extend,
    btree_map_lookup,
    ordmap_insert,
    // ordmap_extend,
    ordmap_lookup,
);

criterion_main!(benches);
