//! near-guard: an on-chain spending policy for AI agents.
//!
//! An owner sets limits for each agent. The agent calls `spend`. The contract
//! checks the policy and moves the NEAR. The contract refuses overspend.
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::{env, near, require, AccountId, NearToken, PanicOnDefault, Promise};

const DAY_MS: u64 = 86_400_000;

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Policy {
    pub per_tx: u128,
    pub daily: u128,
    pub spent_today: u128,
    pub day: u64,
    pub receivers: Vec<AccountId>,
    pub enabled: bool,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    owner: AccountId,
    paused: bool,
    policies: LookupMap<AccountId, Policy>,
}

fn today() -> u64 {
    env::block_timestamp() / 1_000_000 / DAY_MS
}

#[near]
impl Contract {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        Self { owner, paused: false, policies: LookupMap::new(b"p") }
    }

    /// The owner writes or replaces one policy. Call this before the agent runs.
    pub fn set_policy(
        &mut self,
        agent: AccountId,
        per_tx: U128,
        daily: U128,
        receivers: Vec<AccountId>,
    ) {
        require!(env::predecessor_account_id() == self.owner, "only the owner can set a policy");
        require!(per_tx.0 > 0, "per_tx must be greater than zero");
        require!(daily.0 >= per_tx.0, "daily must be at least per_tx");
        self.policies.insert(
            &agent,
            &Policy { per_tx: per_tx.0, daily: daily.0, spent_today: 0, day: today(), receivers, enabled: true },
        );
        env::log_str(&format!("EVENT policy_set agent={} per_tx={} daily={}", agent, per_tx.0, daily.0));
    }

    /// The owner stops or restarts one agent.
    pub fn set_enabled(&mut self, agent: AccountId, enabled: bool) {
        require!(env::predecessor_account_id() == self.owner, "only the owner can change this");
        let mut p = self.policies.get(&agent).expect("agent has no policy");
        p.enabled = enabled;
        self.policies.insert(&agent, &p);
    }

    /// The owner stops every agent.
    pub fn set_paused(&mut self, paused: bool) {
        require!(env::predecessor_account_id() == self.owner, "only the owner can pause");
        self.paused = paused;
    }

    /// The agent asks for money. The contract pays only when the policy allows it.
    #[payable]
    pub fn spend(&mut self, receiver: AccountId, amount: U128) -> Promise {
        let agent = env::predecessor_account_id();
        require!(!self.paused, "all spending is paused");
        require!(env::attached_deposit() == NearToken::from_yoctonear(amount.0), "attach exactly amount");

        let mut p = self.policies.get(&agent).expect("agent has no policy");
        require!(p.enabled, "agent is disabled");
        require!(amount.0 <= p.per_tx, "amount is over the per-transaction limit");
        require!(p.receivers.is_empty() || p.receivers.contains(&receiver), "receiver is not on the allowlist");

        let d = today();
        if p.day != d {
            p.day = d;
            p.spent_today = 0;
        }
        require!(p.spent_today + amount.0 <= p.daily, "amount is over the daily limit");
        p.spent_today += amount.0;
        self.policies.insert(&agent, &p);

        env::log_str(&format!(
            "EVENT spend agent={} receiver={} amount={} spent_today={} daily={}",
            agent, receiver, amount.0, p.spent_today, p.daily
        ));
        Promise::new(receiver).transfer(NearToken::from_yoctonear(amount.0))
    }

    /// Read one policy. Use this in a dashboard.
    pub fn get_policy(&self, agent: AccountId) -> Option<Policy> {
        self.policies.get(&agent).map(|mut p| {
            if p.day != today() { p.spent_today = 0; p.day = today(); }
            p
        })
    }

    pub fn get_owner(&self) -> AccountId { self.owner.clone() }
    pub fn is_paused(&self) -> bool { self.paused }
    pub fn version(&self) -> String { "near-guard 0.1.0".to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{testing_env, test_utils::VMContextBuilder};
    use near_sdk::VMContext;

    fn ctx(predecessor: &str, deposit: u128, ts: u64) -> VMContext {
        VMContextBuilder::new()
            .predecessor_account_id(predecessor.parse().unwrap())
            .attached_deposit(NearToken::from_yoctonear(deposit))
            .block_timestamp(ts * DAY_MS * 1_000_000)
            .build()
    }

    fn setup() -> (Contract, AccountId) {
        testing_env!(ctx("owner.near", 0, 1));
        let agent: AccountId = "agent.near".parse().unwrap();
        let mut c = Contract::new("owner.near".parse().unwrap());
        c.set_policy(agent.clone(), U128(1_000_000), U128(2_500_000), vec!["shop.near".parse().unwrap()]);
        (c, agent)
    }

    #[test]
    fn pays_inside_the_policy() {
        let (mut c, agent) = setup();
        testing_env!(ctx("agent.near", 1_000_000, 10));
        c.spend("shop.near".parse().unwrap(), U128(1_000_000));
        let p = c.get_policy(agent).unwrap();
        assert_eq!(p.spent_today, 1_000_000);
    }

    #[test]
    #[should_panic(expected = "per-transaction limit")]
    fn refuses_a_large_single_payment() {
        let (mut c, _) = setup();
        testing_env!(ctx("agent.near", 2_000_000, 10));
        c.spend("shop.near".parse().unwrap(), U128(2_000_000));
    }

    #[test]
    #[should_panic(expected = "daily limit")]
    fn refuses_when_the_day_is_used_up() {
        let (mut c, _) = setup();
        testing_env!(ctx("agent.near", 1_000_000, 10));
        c.spend("shop.near".parse().unwrap(), U128(1_000_000));
        testing_env!(ctx("agent.near", 1_000_000, 10));
        c.spend("shop.near".parse().unwrap(), U128(1_000_000));
        testing_env!(ctx("agent.near", 1_000_000, 10));
        c.spend("shop.near".parse().unwrap(), U128(1_000_000));
    }

    #[test]
    #[should_panic(expected = "allowlist")]
    fn refuses_an_unknown_receiver() {
        let (mut c, _) = setup();
        testing_env!(ctx("agent.near", 500_000, 10));
        c.spend("evil.near".parse().unwrap(), U128(500_000));
    }

    #[test]
    #[should_panic(expected = "only the owner")]
    fn refuses_a_policy_change_from_an_agent() {
        let (mut c, _) = setup();
        testing_env!(ctx("agent.near", 0, 10));
        c.set_policy("agent.near".parse().unwrap(), U128(9_000_000), U128(9_000_000), vec![]);
    }

    #[test]
    #[should_panic(expected = "paused")]
    fn refuses_spending_when_paused() {
        let (mut c, _) = setup();
        testing_env!(ctx("owner.near", 0, 10));
        c.set_paused(true);
        testing_env!(ctx("agent.near", 100, 10));
        c.spend("shop.near".parse().unwrap(), U128(100));
    }

    #[test]
    fn resets_the_counter_on_a_new_day() {
        let (mut c, agent) = setup();
        testing_env!(ctx("agent.near", 1_000_000, 10));
        c.spend("shop.near".parse().unwrap(), U128(1_000_000));
        testing_env!(ctx("agent.near", 1_000_000, 11));
        c.spend("shop.near".parse().unwrap(), U128(1_000_000));
        assert_eq!(c.get_policy(agent).unwrap().spent_today, 1_000_000);
    }

    #[test]
    #[should_panic(expected = "attach exactly")]
    fn refuses_a_wrong_deposit() {
        let (mut c, _) = setup();
        testing_env!(ctx("agent.near", 999, 10));
        c.spend("shop.near".parse().unwrap(), U128(1_000_000));
    }

    #[test]
    fn disabled_agent_is_refused() {
        let (mut c, agent) = setup();
        testing_env!(ctx("owner.near", 0, 10));
        c.set_enabled(agent, false);
        testing_env!(ctx("agent.near", 1_000, 10));
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            c.spend("shop.near".parse().unwrap(), U128(1_000))
        }));
        assert!(r.is_err());
    }
}
