use anyhow::{bail, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetLimits {
    pub daily_tokens: u64,
    pub monthly_tokens: u64,
    pub per_task_tokens: u64,
    pub per_provider_tokens: u64,
}

impl Default for BudgetLimits {
    fn default() -> Self {
        Self {
            daily_tokens: 100_000,
            monthly_tokens: 2_000_000,
            per_task_tokens: 20_000,
            per_provider_tokens: 100_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextBudget {
    pub max_input_tokens: u64,
    pub max_output_tokens: u64,
    pub max_tool_tokens: u64,
    pub max_search_results: usize,
    pub max_document_chars: usize,
    pub max_history_tokens: u64,
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self {
            max_input_tokens: 12_000,
            max_output_tokens: 4_000,
            max_tool_tokens: 4_000,
            max_search_results: 5,
            max_document_chars: 80_000,
            max_history_tokens: 6_000,
        }
    }
}

#[derive(Debug, Default)]
struct BudgetState {
    daily: HashMap<String, u64>,
    monthly: HashMap<String, u64>,
    task: HashMap<String, u64>,
    provider: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct TokenBudgetLedger {
    limits: BudgetLimits,
    state: Arc<Mutex<BudgetState>>,
}

impl TokenBudgetLedger {
    pub fn new(limits: BudgetLimits) -> Result<Self> {
        if limits.daily_tokens == 0
            || limits.monthly_tokens == 0
            || limits.per_task_tokens == 0
            || limits.per_provider_tokens == 0
        {
            bail!("token budget limits must be positive")
        }
        Ok(Self {
            limits,
            state: Arc::new(Mutex::new(BudgetState::default())),
        })
    }

    pub fn reserve(
        &self,
        task_id: impl Into<String>,
        provider: impl Into<String>,
        input_tokens: u64,
        output_tokens: u64,
    ) -> Result<BudgetReservation> {
        let task_id = task_id.into();
        let provider = provider.into();
        let requested = input_tokens.saturating_add(output_tokens);
        let day = Utc::now().format("%Y-%m-%d").to_string();
        let month = Utc::now().format("%Y-%m").to_string();
        let mut state = self.state.lock().expect("token budget mutex poisoned");

        check_limit(
            "daily",
            state.daily.get(&day).copied().unwrap_or_default(),
            requested,
            self.limits.daily_tokens,
        )?;
        check_limit(
            "monthly",
            state.monthly.get(&month).copied().unwrap_or_default(),
            requested,
            self.limits.monthly_tokens,
        )?;
        check_limit(
            "task",
            state.task.get(&task_id).copied().unwrap_or_default(),
            requested,
            self.limits.per_task_tokens,
        )?;
        check_limit(
            "provider",
            state.provider.get(&provider).copied().unwrap_or_default(),
            requested,
            self.limits.per_provider_tokens,
        )?;

        increment(&mut state.daily, day.clone(), requested);
        increment(&mut state.monthly, month.clone(), requested);
        increment(&mut state.task, task_id.clone(), requested);
        increment(&mut state.provider, provider.clone(), requested);

        Ok(BudgetReservation {
            ledger: self.clone(),
            day,
            month,
            task_id,
            provider,
            reserved: requested,
            settled: false,
        })
    }

    pub fn limits(&self) -> BudgetLimits {
        self.limits
    }
}

fn check_limit(scope: &str, used: u64, requested: u64, limit: u64) -> Result<()> {
    if used.saturating_add(requested) > limit {
        bail!(
            "token budget exceeded for {scope}: requested {requested}, available {}",
            limit.saturating_sub(used)
        )
    }
    Ok(())
}

fn increment(map: &mut HashMap<String, u64>, key: String, amount: u64) {
    let value = map.entry(key).or_default();
    *value = value.saturating_add(amount);
}

pub struct BudgetReservation {
    ledger: TokenBudgetLedger,
    day: String,
    month: String,
    task_id: String,
    provider: String,
    reserved: u64,
    settled: bool,
}

impl BudgetReservation {
    pub fn settle(mut self, input_tokens: u64, output_tokens: u64) -> Result<()> {
        let actual = input_tokens.saturating_add(output_tokens);
        if actual > self.reserved {
            bail!("actual token usage exceeded the reserved budget")
        }
        let refund = self.reserved - actual;
        let mut state = self
            .ledger
            .state
            .lock()
            .expect("token budget mutex poisoned");
        decrement(&mut state.daily, &self.day, refund);
        decrement(&mut state.monthly, &self.month, refund);
        decrement(&mut state.task, &self.task_id, refund);
        decrement(&mut state.provider, &self.provider, refund);
        self.settled = true;
        Ok(())
    }
}

impl Drop for BudgetReservation {
    fn drop(&mut self) {
        if self.settled {
            return;
        }
        let Ok(mut state) = self.ledger.state.lock() else {
            return;
        };
        decrement(&mut state.daily, &self.day, self.reserved);
        decrement(&mut state.monthly, &self.month, self.reserved);
        decrement(&mut state.task, &self.task_id, self.reserved);
        decrement(&mut state.provider, &self.provider, self.reserved);
    }
}

fn decrement(map: &mut HashMap<String, u64>, key: &str, amount: u64) {
    if let Some(value) = map.get_mut(key) {
        *value = value.saturating_sub(amount);
        if *value == 0 {
            map.remove(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> BudgetLimits {
        BudgetLimits {
            daily_tokens: 100,
            monthly_tokens: 100,
            per_task_tokens: 60,
            per_provider_tokens: 80,
        }
    }

    #[test]
    fn reservation_is_atomic_across_all_budget_scopes() {
        let ledger = TokenBudgetLedger::new(limits()).unwrap();
        let reservation = ledger.reserve("task-a", "provider-a", 20, 20).unwrap();
        assert!(ledger.reserve("task-a", "provider-a", 20, 21).is_err());
        reservation.settle(10, 10).unwrap();
        assert!(ledger.reserve("task-a", "provider-a", 20, 20).is_ok());
    }

    #[test]
    fn dropped_reservation_releases_reserved_tokens() {
        let ledger = TokenBudgetLedger::new(limits()).unwrap();
        {
            let _reservation = ledger.reserve("task-a", "provider-a", 40, 20).unwrap();
        }
        assert!(ledger.reserve("task-a", "provider-a", 40, 20).is_ok());
    }

    #[test]
    fn actual_usage_cannot_exceed_reserved_usage() {
        let ledger = TokenBudgetLedger::new(limits()).unwrap();
        let reservation = ledger.reserve("task-a", "provider-a", 10, 10).unwrap();
        assert!(reservation.settle(10, 11).is_err());
    }

    #[test]
    fn context_defaults_are_bounded() {
        let budget = ContextBudget::default();
        assert_eq!(budget.max_search_results, 5);
        assert!(budget.max_history_tokens < budget.max_input_tokens);
    }
}
