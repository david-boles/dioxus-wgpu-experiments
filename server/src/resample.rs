//! Minimum viable signal backend, all in-memory, in-CPU, all pre-computed.
use std::{
    collections::{BTreeMap, HashMap},
    iter,
};

use shared::types::{AbsDiffTime, EmptyContinuousSignalBlock, ResamplePeriod, Samples, Time};

// Must be at least 2 since we need 2 samples to select an initial period for resampling (could be a different threshold but isn't currently).
pub const NUM_SAMPLES_TO_REDUCE_PERIOD_FOR: u8 = 100;
pub const COULD_REALLY_USE_SMALLER_PERIOD_POWER_OF_2: u8 = 8;

struct ResampleBlock<S>(Vec<S>);

struct Resample<S, RS> {
    samples: Vec<(Time, S)>,
    resamples: BTreeMap<ResamplePeriod, Samples<ResampleBlock<S>, EmptyContinuousSignalBlock<S>>>,
    num_new_samples_that_could_really_use_smaller_period: u8,
}

impl<S> Resample<S> {
    pub fn new() -> Self {
        Resample {
            samples: vec![],
            resamples: BTreeMap::new(),
            num_new_samples_that_could_really_use_smaller_period: 0,
        }
    }

    /// Time of the samples must be monotonically increasing
    pub fn ingest_next_sample(&mut self, sample: (Time, S)) {
        let (sample_time, sample) = sample;

        // Validate input
        let prev_sample_time = self.samples.last().map(|(t, _)| *t);
        assert!(prev_sample_time
            .map(|prev| sample_time >= prev)
            .unwrap_or(true));

        // Check for whether we need to reduce the minimum re-sampling period
        let min_sampling_period = self.resamples.first_key_value().map(|(sp, _)| *sp);
        match (prev_sample_time, min_sampling_period) {
            (_, None) => self.num_new_samples_that_could_really_use_smaller_period += 1,
            (Some(prev_sample_time), Some(min_sampling_period)) => {
                if ResamplePeriod::from_diff_rounding_down(sample_time.abs_diff(prev_sample_time))
                    .n_powers_of_2_larger_or_max(COULD_REALLY_USE_SMALLER_PERIOD_POWER_OF_2)
                    < min_sampling_period
                {
                    self.num_new_samples_that_could_really_use_smaller_period += 1;
                }
            }
            _ => unreachable!(
                "a minimum sampling period existing implies at least two samples have been pushed"
            ),
        }

        if self.num_new_samples_that_could_really_use_smaller_period
            >= NUM_SAMPLES_TO_REDUCE_PERIOD_FOR
        {
            let new_sampling_period = if let Some(min_sampling_period) = min_sampling_period {
                // We've already started resampling at some minimum period.
                // Reduce by one, hopefully that will be enough. Re-reducing the sampling rate over and over again is bad, but having a ton of empty blocks (losing resampled block density) are worse.
                min_sampling_period.next_smallest().expect("there must be a smaller available sampling period if samples could use a smaller one than the current minimum")
            } else {
                // Pick an initial minimum resampling period.
                // This is just a heuristic but we don't necessarily want to pick based on the minimum time between samples (could be 0 or occasionally small)
                // Instead, we pick based on the maximum time between samples and add one for good measure (having lots of empty blocks is especially bad).
                // Then there's COULD_REALLY_USE_SMALLER_PERIOD_POWER_OF_2 hysteresis before we start decreasing resampling period from here.

                let existing_sample_times = || self.samples.iter().map(|(t, _)| *t);

                iter::zip(
                    existing_sample_times(),
                    existing_sample_times()
                        .skip(1)
                        .chain(iter::once(sample_time)),
                )
                .map(|(t1, t2)| ResamplePeriod::from_diff_rounding_down(t1.abs_diff(t2)))
                .max()
                .expect("NUM_SAMPLES_TO_REDUCE_PERIOD_FOR must be greater than 2")
            };

            todo!()
        }
    }
}
