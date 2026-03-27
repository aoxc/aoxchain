use std::collections::BTreeMap;

/// Deterministic scheduling decision for a batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulingDecision {
    pub lane_order: Vec<String>,
    pub partition_count: usize,
}

impl SchedulingDecision {
    pub fn serial(lane_order: Vec<String>) -> Self {
        Self {
            lane_order,
            partition_count: 1,
        }
    }
}

/// Deterministic scheduler that creates lane-first execution plans.
#[derive(Debug, Clone)]
pub struct DeterministicScheduler {
    max_partitions: usize,
}

impl DeterministicScheduler {
    pub fn new(max_partitions: usize) -> Self {
        Self {
            max_partitions: max_partitions.max(1),
        }
    }

    /// Produces a stable lane order from input lane counts.
    pub fn plan(&self, lane_counts: &BTreeMap<String, usize>) -> SchedulingDecision {
        let mut lanes: Vec<String> = lane_counts
            .iter()
            .filter_map(|(lane, count)| (*count > 0).then_some(lane.clone()))
            .collect();
        lanes.sort();

        SchedulingDecision {
            partition_count: self.max_partitions.min(lanes.len().max(1)),
            lane_order: lanes,
        }
    }

    /// Buckets pre-sorted lane inputs into deterministic partitions.
    pub fn partition_lane_items<T: Clone>(&self, items: &[T]) -> Vec<Vec<T>> {
        let partition_count = self.max_partitions.min(items.len().max(1));
        let mut partitions = vec![Vec::new(); partition_count];
        for (idx, item) in items.iter().cloned().enumerate() {
            partitions[idx % partition_count].push(item);
        }
        partitions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_is_stable() {
        let scheduler = DeterministicScheduler::new(8);
        let mut counts = BTreeMap::new();
        counts.insert("wasm".to_string(), 3);
        counts.insert("evm".to_string(), 10);
        counts.insert("move".to_string(), 0);

        let decision = scheduler.plan(&counts);
        assert_eq!(decision.lane_order, vec!["evm", "wasm"]);
        assert_eq!(decision.partition_count, 2);
    }

    #[test]
    fn partition_is_stable_and_bounded() {
        let scheduler = DeterministicScheduler::new(3);
        let partitions = scheduler.partition_lane_items(&[1, 2, 3, 4, 5]);

        assert_eq!(partitions.len(), 3);
        assert_eq!(partitions[0], vec![1, 4]);
        assert_eq!(partitions[1], vec![2, 5]);
        assert_eq!(partitions[2], vec![3]);
    }
}
