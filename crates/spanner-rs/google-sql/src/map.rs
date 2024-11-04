pub struct Mapping<const N: usize, K, V> {
    pub default: [(K, V); N],
    pub reverse: [(V, usize); N],
}

impl<const N: usize, K, V> Mapping<N, K, V>
where
    K: Ord,
    V: Ord,
{
    pub const unsafe fn new_assert_sorted(default: [(K, V); N]) -> Self
    where
        V: ~const Ord + Copy,
    {
        let reverse = const_utils::build_reverse_index(&default);

        Self { default, reverse }
    }

    pub const fn new(mut default: [(K, V); N]) -> Self
    where
        K: ~const Ord,
        V: ~const Ord + Copy,
    {
        const_utils::sort_unique(&mut default);
        let reverse = const_utils::build_reverse_index(&default);

        Self { default, reverse }
    }
}

impl<const N: usize, V> Mapping<N, UniCase<&'static str>, V>
where
    V: Ord,
{
    pub fn get(&self, target: &str) -> Option<&V> {
        let index = self.default.binary_search_by(|(s, _)| target.cmp(s)).ok()?;
        Some(&self.default[index].1)
    }

    pub fn get_reverse(&self, target: &V) -> Option<&UniCase<&'static str>> {
        let index = self.reverse.binary_search_by(|(s, _)| target.cmp(s)).ok()?;
        let mapping_index = self.reverse[index].1;
        Some(&self.default[mapping_index].0)
    }

    pub const fn new_unicase(mut default: [(&'static str, V); N]) -> Self
    where
        V: ~const Ord + Copy,
    {
        const fn compare<V>(a: &(&str, V), b: &(&str, V)) -> std::cmp::Ordering {
            const_utils::ascii_cmp_ignore_case(a.0, b.0)
        }

        const fn build_pair<V: Copy>(
            _: usize,
            p: &(&'static str, V),
        ) -> (UniCase<&'static str>, V) {
            (UniCase::unicode(p.0), p.1)
        }

        const_utils::sort_unique_by(&mut default, compare);
        let default = const_utils::array_ref_map(&default, build_pair);

        let reverse = const_utils::build_reverse_index(&default);

        Self { default, reverse }
    }
}

macro_rules! mapping {
    ($($k:literal => $v:expr),* $(,)?) => {{
        $crate::map::Mapping::new_unicase([ $(($k, $v)),* ])
    }};
}

pub(crate) use mapping;
use unicase::UniCase;
