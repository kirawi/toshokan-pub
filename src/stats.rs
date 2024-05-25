/*
    - Requirements:
        - Data points should fall within buckets of single days in UTC
        - It should be able to have multiple different kinds of data being recorded
        - Should be able to be shared across threads
        - Should not block server
*/

use chrono::{Datelike, Days, Utc};

/// Container for all statistics per object
pub struct Statistics {
    views: DataBucketVec<usize>,
}

/// A bucket where all datapoints per day are stored.
pub struct DataBucketVec<T> {
    offset: u32,
    // (DayOffset, T)
    data: Vec<(u32, T)>,
}

impl<T> Default for DataBucketVec<T> {
    fn default() -> Self {
        // SAFETY: UTC > CE
        let offset = Utc::now().num_days_from_ce() as u32;
        Self {
            offset,
            data: vec![],
        }
    }
}

impl Statistics {
    pub fn add_view(&mut self) {
        // SAFETY: UTC > CE
        let time = Utc::now().num_days_from_ce() as u32;
        let day_offset = time - self.views.offset;

        if let Some(bucket) = self
            .views
            .data
            .last_mut()
            .filter(|(last_day_offset, _)| *last_day_offset == day_offset)
        {
            bucket.1 += 1;
        } else {
            self.views.data.push((day_offset, 1));
        }
    }
}
