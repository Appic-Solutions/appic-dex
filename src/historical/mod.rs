// a module for keeping track of historical data in different time pre aggregated buckets to make
// it easier to show to useres in appic explorer
// Stored data include swap_volume, fee generation, price movement, and liquidty changes
// Time buckets include Ten minutes ,Hourly, Daily, Monthly, and Yearly

// to prevent storage overflow a mechanism automatically removes old data
// historical storage keeps at max 144 ten minute buckets, 72 hourly bucket, 90 daily bucket, 36 monthly bucket, and 12 yealry
// buckets

use crate::{
    pool::types::PoolState,
    state::{mutate_state, read_state},
};
use types::{HistoryBucket, PoolHistory};

pub mod types;

/// Maximum number of buckets for each timeframe.
const MAX_BUCKETS: [(TimeFrame, usize); 4] = [
    (TimeFrame::Hourly, 48),
    (TimeFrame::Daily, 60),
    (TimeFrame::Monthly, 24),
    (TimeFrame::Yearly, 10),
];

/// Timeframe buckets for historical data.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TimeFrame {
    Hourly,
    Daily,
    Monthly,
    Yearly,
}

impl PoolHistory {
    /// Get the vector for a specific timeframe.
    fn get_frame_mut(&mut self, timeframe: TimeFrame) -> &mut Vec<HistoryBucket> {
        match timeframe {
            TimeFrame::Hourly => &mut self.hourly_frame,
            TimeFrame::Daily => &mut self.daily_frame,
            TimeFrame::Monthly => &mut self.monthly_frame,
            TimeFrame::Yearly => &mut self.yearly_frame,
        }
    }

    /// Get the maximum number of buckets for a timeframe.
    fn get_max_buckets(timeframe: TimeFrame) -> usize {
        MAX_BUCKETS
            .iter()
            .find(|&&(t, _)| t == timeframe)
            .map(|&(_, max)| max)
            .unwrap_or(0)
    }
}

/// Captures historical data for all pools and timeframes.
pub fn capture_historical_data() {
    let timestamp_nanos = ic_cdk::api::time();
    let timestamp_secs = nanos_to_seconds(timestamp_nanos);

    let pools = read_state(|s| s.get_pools());

    for (pool_id, pool_state) in pools {
        let mut pool_history = read_state(|s| s.get_pool_history(&pool_id));

        // Capture data for all timeframes
        for timeframe in [
            TimeFrame::Hourly,
            TimeFrame::Daily,
            TimeFrame::Monthly,
            TimeFrame::Yearly,
        ] {
            capture_bucket(&mut pool_history, &pool_state, timestamp_secs, timeframe);
        }

        // Update state with modified history (assuming a write_state function exists)
        mutate_state(|s| s.set_pool_history(pool_id, pool_history));
    }
}

/// Captures a bucket for a specific timeframe.
fn capture_bucket(
    pool_history: &mut PoolHistory,
    pool_state: &PoolState,
    timestamp: u64,
    timeframe: TimeFrame,
) {
    let (start_timestamp, end_timestamp) = calculate_start_and_end_timestamp(timestamp, &timeframe);
    let frame = pool_history.get_frame_mut(timeframe);

    if let Some(bucket) = frame.last_mut() {
        // Update existing bucket if it matches the timeframe
        if bucket.start_timestamp == start_timestamp && bucket.end_timestamp == end_timestamp {
            update_bucket(bucket, pool_state);
        } else {
            // Create new bucket
            frame.push(create_bucket(start_timestamp, end_timestamp, pool_state));
        }
    } else {
        // Initialize first bucket
        frame.push(create_bucket(start_timestamp, end_timestamp, pool_state));
    }

    // Limit the number of buckets
    limit_vec_length(frame, PoolHistory::get_max_buckets(timeframe));
}

/// Creates a new HistoryBucket from pool state.
fn create_bucket(
    start_timestamp: u64,
    end_timestamp: u64,
    pool_state: &PoolState,
) -> HistoryBucket {
    HistoryBucket {
        start_timestamp,
        end_timestamp,
        swap_volume_token0_start: pool_state.swap_volume0_all_time,
        swap_volume_token0_end: pool_state.swap_volume0_all_time,
        swap_volume_token1_start: pool_state.swap_volume1_all_time,
        swap_volume_token1_end: pool_state.swap_volume1_all_time,
        fee_generated_token0_start: pool_state.generated_swap_fee0,
        fee_generated_token0_end: pool_state.generated_swap_fee0,
        fee_generated_token1_start: pool_state.generated_swap_fee1,
        fee_generated_token1_end: pool_state.generated_swap_fee1,
        token0_reserves_start: pool_state.pool_reserve0,
        token0_reserves_end: pool_state.pool_reserve0,
        token1_reserves_start: pool_state.pool_reserve1,
        token1_reserves_end: pool_state.pool_reserve1,
        last_sqrtx96_price: pool_state.sqrt_price_x96,
        inrange_liquidity: pool_state.liquidity,
        active_tick: pool_state.tick,
    }
}

/// Updates an existing HistoryBucket with current pool state.
fn update_bucket(bucket: &mut HistoryBucket, pool_state: &PoolState) {
    bucket.swap_volume_token0_end = pool_state.swap_volume0_all_time;
    bucket.swap_volume_token1_end = pool_state.swap_volume1_all_time;
    bucket.fee_generated_token0_end = pool_state.generated_swap_fee0;
    bucket.fee_generated_token1_end = pool_state.generated_swap_fee1;
    bucket.token0_reserves_end = pool_state.pool_reserve0;
    bucket.token1_reserves_end = pool_state.pool_reserve1;
    bucket.last_sqrtx96_price = pool_state.sqrt_price_x96;
    bucket.inrange_liquidity = pool_state.liquidity;
    bucket.active_tick = pool_state.tick;
}

/// Calculates start and end timestamps for a bucket, aligned to the timeframe.
fn calculate_start_and_end_timestamp(timestamp: u64, timeframe: &TimeFrame) -> (u64, u64) {
    let start_time = align_timestamp_to_bucket(timestamp, timeframe);
    let duration_secs = match timeframe {
        TimeFrame::Hourly => 60 * 60,            // 1 hour
        TimeFrame::Daily => 24 * 60 * 60,        // 1 day
        TimeFrame::Monthly => 30 * 24 * 60 * 60, // Approx 30 days
        TimeFrame::Yearly => 365 * 24 * 60 * 60, // Approx 365 days
    };
    (start_time, start_time + duration_secs)
}

/// Aligns a timestamp to the start of a bucket.
fn align_timestamp_to_bucket(timestamp: u64, timeframe: &TimeFrame) -> u64 {
    match timeframe {
        TimeFrame::Hourly => timestamp - (timestamp % (60 * 60)),
        TimeFrame::Daily => timestamp - (timestamp % (24 * 60 * 60)),
        TimeFrame::Monthly => {
            // Simplified: Align to nearest 30-day period
            timestamp - (timestamp % (30 * 24 * 60 * 60))
        }
        TimeFrame::Yearly => {
            // Simplified: Align to nearest 365-day period
            timestamp - (timestamp % (365 * 24 * 60 * 60))
        }
    }
}

/// Converts epoch time from nanoseconds to seconds.
fn nanos_to_seconds(epoch_nanos: u64) -> u64 {
    epoch_nanos / 1_000_000_000
}

/// Limits the length of a vector to `max_len` by removing elements from the front.
fn limit_vec_length<T>(vec: &mut Vec<T>, max_len: usize) {
    if vec.len() > max_len {
        vec.drain(..vec.len() - max_len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_alignment() {
        let timestamp = 1677655321_u64; // Arbitrary timestamp
        assert_eq!(
            align_timestamp_to_bucket(timestamp, &TimeFrame::Hourly),
            1677654000
        ); // Aligns to nearest hour
        assert_eq!(
            align_timestamp_to_bucket(timestamp, &TimeFrame::Daily),
            1677628800
        ); // Aligns to nearest day
        assert_eq!(
            align_timestamp_to_bucket(timestamp, &TimeFrame::Monthly),
            1677024000
        ); // Aligns to nearest 30-day
        assert_eq!(
            align_timestamp_to_bucket(timestamp, &TimeFrame::Yearly),
            1671408000
        ); // Aligns to nearest 365-day
    }

    #[test]
    fn test_limit_vec_length() {
        let mut vec: Vec<i32> = (0..200).collect();
        limit_vec_length(&mut vec, 144);
        assert_eq!(vec.len(), 144);
        assert_eq!(vec[0], 56); // First 56 elements removed
        assert_eq!(vec[143], 199); // Last element preserved
    }

    #[test]
    fn test_nanos_to_seconds() {
        assert_eq!(nanos_to_seconds(1677654321000000000), 1677654321);
        assert_eq!(nanos_to_seconds(0), 0);
    }
}
