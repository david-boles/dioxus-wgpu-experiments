use std::collections::{BTreeMap, BTreeSet};

/// Single (easier for interoperability), signed (for completely arbitrary choice of reference 0) int that's wide enough for all imaginable usecases.
///
/// Reference 0 is context-defined, times since Unix epoch, the start of a trace, etc.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    nanoseconds: i128,
}

impl Time {
    const MAX: Self = Self {
        nanoseconds: i128::MAX,
    };
    const MIN: Self = Self {
        nanoseconds: i128::MIN,
    };

    pub fn abs_diff(&self, other: Self) -> AbsDiffTime {
        AbsDiffTime {
            nanoseconds: self.nanoseconds.abs_diff(other.nanoseconds),
        }
    }
}

#[test]
fn time() {
    assert!(Time::MAX == Time::MAX);
    assert!(Time::MAX > Time::MIN);
}

pub struct AbsDiffTime {
    nanoseconds: u128,
}

impl AbsDiffTime {
    const ZERO: AbsDiffTime = AbsDiffTime { nanoseconds: 0 };
}

/// Raw samples are resampled to a fixed frequency of the form 2^n ns.
///
/// This period is the inverse of that frequency
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResamplePeriod {
    power_of_two: u8,
}

impl ResamplePeriod {
    const MIN: Self = Self { power_of_two: 0 };
    // 2^126 is the greatest power of two representable by i128s
    const MAX: Self = Self { power_of_two: 126 };

    pub fn from_diff_rounding_down(diff: AbsDiffTime) -> Self {
        Self {
            power_of_two: u8::try_from(u128::checked_ilog2(diff.nanoseconds).unwrap_or(0))
                .unwrap()
                .min(Self::MAX.power_of_two),
        }
    }

    pub fn next_smallest(&self) -> Option<ResamplePeriod> {
        self.power_of_two
            .checked_sub(1)
            .map(|power_of_two| ResamplePeriod { power_of_two })
    }

    pub fn n_powers_of_2_larger_or_max(&self, n: u8) -> ResamplePeriod {
        ResamplePeriod {
            power_of_two: self
                .power_of_two
                .saturating_add(n)
                .min(Self::MAX.power_of_two),
        }
    }
}

#[test]
fn resample_period() {
    assert_eq!(
        ResamplePeriod::from_diff_rounding_down(Time::MAX.abs_diff(Time::MIN)),
        ResamplePeriod::MAX
    );
    assert_eq!(
        ResamplePeriod::from_diff_rounding_down(Time::MAX.abs_diff(Time::MAX)),
        ResamplePeriod::MIN
    );
}

/// Indexes into both raw data samples and resampled data in a [SampleBlockTree].
///
/// Time when raw samples were taken must monotonically increase with sample index.
///
/// For resampled data samples, the index times the resampling period equals the minimum time covered by that sample.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SampleIndex {
    index: i128,
}

impl SampleIndex {
    pub const MIN: Self = Self { index: i128::MIN };
    pub const MAX: Self = Self { index: i128::MAX };
    pub fn resampled_index_for_time(period: ResamplePeriod, time: Time) -> Self {
        Self {
            index: time.nanoseconds.div_euclid(1 << period.power_of_two),
        }
    }
}

#[test]
fn test_sample_index() {
    assert_eq!(
        SampleIndex::resampled_index_for_time(ResamplePeriod::MIN, Time { nanoseconds: -1 }),
        SampleIndex { index: -1 }
    );
    assert_eq!(
        SampleIndex::resampled_index_for_time(ResamplePeriod::MIN, Time { nanoseconds: 0 }),
        SampleIndex { index: 0 }
    );
    assert_eq!(
        SampleIndex::resampled_index_for_time(ResamplePeriod::MIN, Time { nanoseconds: 1 }),
        SampleIndex { index: 1 }
    );

    const period_2ns: ResamplePeriod = ResamplePeriod { power_of_two: 1 };
    for (index, times) in [
        (-2, vec![-3]),
        (-1, vec![-2, -1]),
        (0, vec![0, 1]),
        (1, vec![2, 3]),
        (2, vec![4]),
    ] {
        for time in times {
            assert_eq!(
                SampleIndex::resampled_index_for_time(period_2ns, Time { nanoseconds: time }),
                SampleIndex { index }
            );
        }
    }

    assert_eq!(
        SampleIndex::resampled_index_for_time(ResamplePeriod::MAX, Time::MIN),
        SampleIndex { index: -2 }
    );
    assert_eq!(
        SampleIndex::resampled_index_for_time(ResamplePeriod::MAX, Time { nanoseconds: 0 }),
        SampleIndex { index: 0 }
    );
    assert_eq!(
        SampleIndex::resampled_index_for_time(ResamplePeriod::MAX, Time::MAX),
        SampleIndex { index: 1 }
    );
}

// /// A non-empty half-open range of sample indices.
// pub struct NonEmptySampleIndexRange {
//     min: SampleIndex,
//     end: SampleIndex,
// }

// impl NonEmptySampleIndexRange {
//     pub fn new(min: SampleIndex, end: SampleIndex) -> Option<Self> {
//         if min.index <= end.index {
//             Some(Self { min, end })
//         } else {
//             None
//         }
//     }
// }

pub enum Block<SB, E> {
    /// A packed block of the samples for some range of indices.
    Samples(SB),
    /// There are no samples for some range of indices.
    Empty(E),
    /// Samples for some range of indices are either permanently or temporarily unavailable.
    Unavailable,
}

/// Type for empty blocks in sample trees for "continuous" signals.
/// Provides the samples on either side of the empty block to render an interpolation between.
pub struct EmptyContinuousSignalBlock<S> {
    pub prev: (Time, S),
    pub next: (Time, S),
}

/// For some signal, provides access to all the, either raw or resampled at some period, samples.
/// It does this in optionally sparse, but otherwise packed blocks.
///
/// The representation of a packed range of samples is generic.
/// They could be vectors, references to portions of a file, GPU buffers, etc.
///
/// In general, a packed representation of samples is efficient and useful for indexing into quickly, rendering in one pass, etc.
/// This is primarily an optimization for signals that might be "bursty": e.g. big data transfers that happen for brief periods of time.
/// In addition it's a helpful common abstraction for whenever packed representations can't be arbitrarily large: e.g. single-signal blocks in a file that a multiple signals are being logged into or GPU buffers that might need to be removed from the local cache with a little granularity.
///
/// Invariants:
/// - A (non-unique) [SampleBlock] exists for every [SampleIndex].
/// - [SampleBlock::Empty] and [SampleBlock::Unavailable] blocks are coalesced with themselves respectively. Two won't be right next to each other.
pub struct Samples<SB, E> {
    /// Keys are the minumum (inclusive) range of each block. Each block either extends to the next key (non-inclusive) or [SampleIndex::MAX] if there is no next block.
    tree: BTreeMap<SampleIndex, Block<SB, E>>,
}

impl<SB, E> Samples<SB, E> {
    pub fn new() -> Self {
        let mut tree = BTreeMap::new();
        tree.insert(SampleIndex::MIN, Block::Unavailable);
        Self { tree }
    }

    /// Get the index of the last non-[SampleBlock::Unavailable] sample.
    pub fn last_available_index(&self) -> Option<SampleIndex> {
        let (min, block) = self.tree.last_key_value().expect("never empty");
        match block {
            Block::Unavailable => {
                match *min {
                    // The unavailable range spans the entire space
                    SampleIndex::MIN => None,
                    // The minimum index of the trailing block of unavailable indices preceeds an available index
                    index => Some(SampleIndex {
                        index: index.index - 1,
                    }),
                }
            }
            // There is no unavailable block at the end, the last index is available.
            Block::Samples(_) | Block::Empty(_) => Some(SampleIndex::MAX),
        }
    }
}
