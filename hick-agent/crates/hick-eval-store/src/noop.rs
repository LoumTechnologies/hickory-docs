use crate::backend::StorageBackend;
use crate::record::EvalRecord;
use anyhow::Result;
use std::sync::Mutex;

/// In-memory backend for tests — never touches disk or git.
#[derive(Default)]
pub struct NoopBackend {
    records: Mutex<Vec<EvalRecord>>,
}

impl NoopBackend {
    pub fn new() -> Self {
        Self::default()
    }

    /// Take all records out of the backend, leaving it empty.
    pub fn drain_records(&self) -> Vec<EvalRecord> {
        std::mem::take(&mut self.records.lock().unwrap())
    }
}

impl StorageBackend for NoopBackend {
    fn append(&self, record: &EvalRecord) -> Result<()> {
        self.records.lock().unwrap().push(record.clone());
        Ok(())
    }

    fn read_all(&self) -> Result<Vec<EvalRecord>> {
        Ok(self.records.lock().unwrap().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{EvalRecord, EvalResultRecord, PruneRecord};
    use chrono::Utc;

    fn eval(machine_id: &str) -> EvalRecord {
        EvalRecord::EvalResult(EvalResultRecord {
            machine_id: machine_id.to_string(),
            timestamp: Utc::now(),
            config_name: "cfg".to_string(),
            query: None,
            response: None,
            scores: None,
            token_cost: None,
            original_turn_id: None,
            wall_secs: None,
            rework_count: None,
        })
    }

    #[test]
    fn append_then_read_all() {
        let b = NoopBackend::new();
        b.append(&eval("m1")).unwrap();
        let all = b.read_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].record_type(), "eval_result");
    }

    #[test]
    fn multiple_appends_accumulate() {
        let b = NoopBackend::new();
        for i in 0..5 {
            b.append(&eval(&format!("m{i}"))).unwrap();
        }
        assert_eq!(b.read_all().unwrap().len(), 5);
    }

    #[test]
    fn prune_record_stored_alongside_eval_records() {
        let b = NoopBackend::new();
        b.append(&eval("m1")).unwrap();
        b.append(&EvalRecord::Prune(PruneRecord {
            machine_id: "m1".to_string(),
            timestamp: Utc::now(),
            config_name: Some("cfg".to_string()),
            reason: Some("old".to_string()),
            pruned_before: None,
            scores: None,
            confidence: None,
            trigger: Some("manual".to_string()),
        }))
        .unwrap();
        let all = b.read_all().unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[1].record_type(), "prune");
    }

    #[test]
    fn drain_records_empties_backend() {
        let b = NoopBackend::new();
        b.append(&eval("m1")).unwrap();
        let drained = b.drain_records();
        assert_eq!(drained.len(), 1);
        assert_eq!(b.read_all().unwrap().len(), 0);
    }
}
