use std::{
    cmp::{max, min},
    collections::VecDeque,
    mem,
};

use js_sys::Math::floor;
use web_sys::console::assert;

fn f32_min(a: f32, b: f32) -> f32 {
    match (a.is_finite(), b.is_finite()) {
        (true, true) => {
            if a.lt(&b) {
                a
            } else {
                b
            }
        }
        (true, false) => a,
        (false, true) => b,
        (false, false) => f32::MAX,
    }
}

fn f32_max(a: f32, b: f32) -> f32 {
    match (a.is_finite(), b.is_finite()) {
        (true, true) => {
            if a.gt(&b) {
                a
            } else {
                b
            }
        }
        (true, false) => a,
        (false, true) => b,
        (false, false) => f32::MAX,
    }
}

#[derive(Debug)]
struct PositionalVecDeque<T> {
    offset_to_position: i128,
    vec_deque: VecDeque<T>,
}

impl<T> PositionalVecDeque<T> {
    fn new() -> Self {
        PositionalVecDeque {
            offset_to_position: 0,
            vec_deque: VecDeque::<T>::new(),
        }
    }

    fn min_pos(&self) -> Option<i128> {
        if self.vec_deque.is_empty() {
            None
        } else {
            Some(self.offset_to_position)
        }
    }

    /// Sets the value at the given position.
    ///
    /// Returns an error if setting the given position would make the stored positions non-continguous.
    fn set_pos(&mut self, pos: i128, value: T) -> Result<(), ()> {
        if self.vec_deque.is_empty() {
            self.offset_to_position = pos;
            self.vec_deque.push_back(value);
            return Ok(());
        } else {
            let index = pos - self.offset_to_position;
            if let Some(element) = self.vec_deque.get_mut(index as usize) {
                *element = value;
                return Ok(());
            } else if index == -1 {
                self.vec_deque.push_front(value);
                self.offset_to_position -= 1;
                return Ok(());
            } else if index == self.vec_deque.len() as i128 {
                self.vec_deque.push_back(value);
                return Ok(());
            }
        }
        Err(())
    }

    fn get_pos(&self, pos: i128) -> Option<&T> {
        let index = pos - self.offset_to_position;
        if !index.is_negative() {
            return self.vec_deque.get(index as usize);
        }
        None
    }

    fn get_pos_mut(&mut self, pos: i128) -> Option<&mut T> {
        let index = pos - self.offset_to_position;
        if !index.is_negative() {
            return self.vec_deque.get_mut(index as usize);
        }
        None
    }

    fn last(&self) -> Option<&T> {
        self.vec_deque.back()
    }

    fn iter_inds(&self) -> impl Iterator<Item = i128> {
        self.offset_to_position..(self.offset_to_position + (self.vec_deque.len() as i128))
    }
}

#[test]
fn test_positional_vec_deque() {
    let mut pvd = PositionalVecDeque::new();
    assert_eq!(pvd.min_pos(), None);
    assert_eq!(pvd.last(), None);
    assert_eq!(pvd.iter_inds().collect::<Vec<_>>(), vec![]);

    pvd.set_pos(10, 10).unwrap();
    assert_eq!(pvd.min_pos(), Some(10));
    assert_eq!(pvd.get_pos(10).map(|p| *p), Some(10));
    assert_eq!(pvd.last().map(|p| *p), Some(10));
    assert_eq!(pvd.iter_inds().collect::<Vec<_>>(), vec![10]);

    pvd.set_pos(9, 9).unwrap();
    pvd.set_pos(11, 11).unwrap();
    assert_eq!(pvd.min_pos(), Some(9));
    assert_eq!(pvd.get_pos(9).map(|p| *p), Some(9));
    assert_eq!(pvd.get_pos(10).map(|p| *p), Some(10));
    assert_eq!(pvd.get_pos(11).map(|p| *p), Some(11));
    assert_eq!(pvd.get_pos(8).map(|p| *p), None);
    assert_eq!(pvd.get_pos(12).map(|p| *p), None);
    assert_eq!(pvd.last().map(|p| *p), Some(11));

    assert_eq!(pvd.set_pos(7, 7), Err(()));
    assert_eq!(pvd.set_pos(13, 13), Err(()));

    assert_eq!(pvd.iter_inds().collect::<Vec<_>>(), vec![9, 10, 11]);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct TimeScale {
    one_s_exp: i32,
}

impl TimeScale {
    fn scale_wider_than_period(period: f32) -> TimeScale {
        TimeScale {
            one_s_exp: period.log2().ceil() as i32,
        }
    }

    fn scale_idx(&self) -> i32 {
        self.one_s_exp
    }

    fn from_scale_idx(scale_idx: i32) -> TimeScale {
        TimeScale {
            one_s_exp: scale_idx,
        }
    }

    fn index_for_time(&self, time: f32) -> i128 {
        ((time as f64) / (2_f64.powi(self.one_s_exp))).floor() as i128
    }

    fn time_centered_in_time_index_period(&self, time_idx: i128) -> f32 {
        (((time_idx as f64) + 0.5) * (2_f64.powi(self.one_s_exp))) as f32
    }
}

#[test]
fn test_time_scale() {
    assert_eq!(
        TimeScale::scale_wider_than_period(0.99),
        TimeScale { one_s_exp: 0 }
    );
    assert_eq!(
        TimeScale::scale_wider_than_period(1.01),
        TimeScale { one_s_exp: 1 }
    );
    assert_eq!(
        TimeScale::scale_wider_than_period(0.49),
        TimeScale { one_s_exp: -1 }
    );
}

#[derive(Clone, Copy, Debug)]
struct Sample {
    /// Seconds
    time: f32,
    value: f32,
}

#[derive(Debug)]
struct DecimationSample {
    num_samples: i128,
    min: f32,
    max: f32,
    last: Option<Sample>,
}

impl DecimationSample {
    fn new() -> Self {
        Self {
            num_samples: 0,
            min: f32::MAX,
            max: f32::MIN,
            last: None,
        }
    }
}

#[derive(Debug)]
struct IncrementalDecimationSpace {
    /// First index by TimeScale, second by offset in time.
    finalized_decimation_samples: PositionalVecDeque<PositionalVecDeque<DecimationSample>>,
    /// Index by TimeScale
    incomplete_decimation_samples: PositionalVecDeque<(i128, DecimationSample)>,
    /// The actual samples, ordered by time
    samples: Vec<Sample>,
}

impl IncrementalDecimationSpace {
    pub fn new() -> Self {
        IncrementalDecimationSpace {
            finalized_decimation_samples: PositionalVecDeque::new(),
            incomplete_decimation_samples: PositionalVecDeque::new(),
            samples: Vec::new(),
        }
    }

    pub fn push_sample(&mut self, sample: Sample) {
        if self.samples.len() > 0 {
            let prev_sample = self.samples.last().unwrap();

            // Make sure we're decimating finely enough
            let delta_t = sample.time - prev_sample.time;
            assert!(delta_t >= 0.0);
            // TODO: smarter handling for reducing decimation scale?
            if delta_t > 0.000001 {
                self.begin_decimating_down_to_at_least_scale(TimeScale::scale_wider_than_period(
                    delta_t,
                ));
            }

            // Process the sample
            for scale_idx in self.finalized_decimation_samples.iter_inds() {
                Self::process_new_sample_for_scale(
                    TimeScale::from_scale_idx(scale_idx as i32),
                    self.finalized_decimation_samples
                        .get_pos_mut(scale_idx)
                        .unwrap(),
                    self.incomplete_decimation_samples
                        .get_pos_mut(scale_idx)
                        .unwrap(),
                    &sample,
                );
            }
            self.samples.push(sample);
        } else {
            // If we only have one sample we can't really know how to decimate it yet.
            self.samples.push(sample)
        }
    }

    /// Only usable once at least one sample has been pushed.
    fn begin_decimating_down_to_at_least_scale(&mut self, scale: TimeScale) {
        dbg!(scale);
        let scale_idx = scale.scale_idx() as i128;
        let new_scale_idxs = match dbg!(self.finalized_decimation_samples.min_pos()) {
            None => scale_idx..(scale_idx + 1),
            Some(current_min_scale_idx) if (scale.scale_idx() as i128) < current_min_scale_idx => {
                dbg!(scale_idx)..dbg!(current_min_scale_idx)
            }
            _ => return,
        };

        // "replay" decimating the samples at these new scales
        for new_scale_idx in new_scale_idxs.rev() {
            dbg!(new_scale_idx);
            let new_scale = TimeScale::from_scale_idx(new_scale_idx as i32);
            let initial_time_index = new_scale.index_for_time(self.samples.first().unwrap().time);
            self.incomplete_decimation_samples
                .set_pos(new_scale_idx, (initial_time_index, DecimationSample::new()))
                .unwrap();
            self.finalized_decimation_samples
                .set_pos(new_scale_idx, PositionalVecDeque::new())
                .unwrap();

            for sample in &self.samples {
                Self::process_new_sample_for_scale(
                    new_scale,
                    self.finalized_decimation_samples
                        .get_pos_mut(new_scale_idx)
                        .unwrap(),
                    self.incomplete_decimation_samples
                        .get_pos_mut(new_scale_idx)
                        .unwrap(),
                    sample,
                );
            }

            // TODO logic for increasing decimation range
        }
    }

    fn process_new_sample_for_scale(
        scale: TimeScale,
        finalized_decimation_samples: &mut PositionalVecDeque<DecimationSample>,
        incomplete_decimation_sample: &mut (i128, DecimationSample),
        sample: &Sample,
    ) {
        let sample_time_index = scale.index_for_time(sample.time);
        assert!(sample_time_index >= incomplete_decimation_sample.0);

        // Finalize decimation samples for all time indices that preceed the new sample's
        if sample_time_index > incomplete_decimation_sample.0 {
            let mut new_d_sample_to_finalize = mem::replace(
                incomplete_decimation_sample,
                (sample_time_index, DecimationSample::new()),
            );

            // Janky-ass logic to interpolate when the decimation sampling rate is higher than the samples are actually coming in
            let num_additional_empty_d_samples =
                sample_time_index - (new_d_sample_to_finalize.0 + 1);

            if new_d_sample_to_finalize.1.num_samples == 0 || num_additional_empty_d_samples > 0 {
                let last_known_sample =
                    new_d_sample_to_finalize
                        .1
                        .last
                        .or(finalized_decimation_samples
                            .last()
                            .map(|lds| lds.last)
                            .flatten());

                if let Some(last_known_sample) = last_known_sample {
                    let interp = |time_idx| -> f32 {
                        (((sample.value - last_known_sample.value)
                            / (sample.time - last_known_sample.time))
                            * (scale.time_centered_in_time_index_period(time_idx)
                                - last_known_sample.time))
                            + last_known_sample.value
                    };

                    new_d_sample_to_finalize.1.min = interp(new_d_sample_to_finalize.0);
                    new_d_sample_to_finalize.1.max = interp(new_d_sample_to_finalize.0);
                    finalized_decimation_samples
                        .set_pos(new_d_sample_to_finalize.0, new_d_sample_to_finalize.1)
                        .unwrap();

                    for time_idx in
                        (sample_time_index - num_additional_empty_d_samples)..sample_time_index
                    {
                        let mut d_sample = DecimationSample::new();
                        d_sample.min = interp(time_idx);
                        d_sample.max = interp(time_idx);
                        finalized_decimation_samples
                            .set_pos(time_idx, d_sample)
                            .unwrap();
                    }
                } else {
                    finalized_decimation_samples
                        .set_pos(new_d_sample_to_finalize.0, new_d_sample_to_finalize.1)
                        .unwrap();

                    for time_idx in
                        (sample_time_index - num_additional_empty_d_samples)..sample_time_index
                    {
                        let d_sample = DecimationSample::new();
                        finalized_decimation_samples
                            .set_pos(time_idx, d_sample)
                            .unwrap();
                    }
                }
            }
        }

        incomplete_decimation_sample.1.num_samples += 1;
        incomplete_decimation_sample.1.min =
            f32_min(sample.value, incomplete_decimation_sample.1.min);
        incomplete_decimation_sample.1.max =
            f32_max(sample.value, incomplete_decimation_sample.1.max);
        incomplete_decimation_sample.1.last = Some(*sample);
    }
}

#[test]
fn test_incremental_decimation() {
    let mut d = IncrementalDecimationSpace::new();
    d.push_sample(Sample {
        time: 10.0,
        value: 10.0,
    });
    d.push_sample(Sample {
        time: 11.0,
        value: 11.0,
    });
    d.push_sample(Sample {
        time: 11.1,
        value: 11.1,
    });
    d.push_sample(Sample {
        time: 11.1,
        value: 11.11,
    });
    d.push_sample(Sample {
        time: 15.0,
        value: 15.0,
    });

    dbg!(d);
}
