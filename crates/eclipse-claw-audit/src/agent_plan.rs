//! Strict import contract for bounded plans produced by Eclipse AI Hub.
//!
//! This module validates data only. It never grants execution permissions and
//! deliberately rejects unknown fields so a newer or wider plan fails closed.

use serde::{Deserialize, Serialize};

use crate::AuditError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AgentRoleId {
    ProcessAnalyst,
    EvidenceReviewer,
    ClaimAuditor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentRole {
    pub id: AgentRoleId,
    pub responsibility: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StopConditionCode {
    BudgetExhausted,
    PermissionDenied,
    EvidenceMissing,
    Timeout,
    HumanDecisionRequired,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DenyPermission {
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentPermissions {
    pub network: DenyPermission,
    pub filesystem_write: DenyPermission,
    pub external_actions: DenyPermission,
    pub secrets: DenyPermission,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentBudget {
    pub max_steps: u32,
    pub max_model_calls: u32,
    pub max_input_tokens: u32,
    pub max_output_tokens: u32,
    pub max_cost_usd: f64,
    pub timeout_ms: u64,
    pub max_parallel_agents: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentRunPlan {
    pub schema_version: String,
    pub plan_id: String,
    pub execution_allowed: bool,
    pub roles: Vec<AgentRole>,
    pub budget: AgentBudget,
    pub permissions: AgentPermissions,
    pub stop_condition_codes: Vec<StopConditionCode>,
    pub stop_conditions: Vec<String>,
}

impl AgentRunPlan {
    pub fn from_json(input: &str) -> Result<Self, AuditError> {
        let plan: Self = serde_json::from_str(input)?;
        plan.validate()?;
        Ok(plan)
    }

    pub fn validate(&self) -> Result<(), AuditError> {
        if self.schema_version != "eclipse.agent-run-plan.v1" {
            return invalid("unsupported agent plan schema");
        }
        if !valid_plan_id(&self.plan_id) {
            return invalid("agent plan id is invalid");
        }
        if self.execution_allowed {
            return invalid("imported agent plans must deny execution");
        }

        let expected_roles = [
            AgentRoleId::ProcessAnalyst,
            AgentRoleId::EvidenceReviewer,
            AgentRoleId::ClaimAuditor,
        ];
        if self.roles.len() != expected_roles.len()
            || self.roles.iter().map(|role| role.id).ne(expected_roles)
        {
            return invalid("agent plan roles must match the fixed allowlist");
        }
        if self
            .roles
            .iter()
            .any(|role| !valid_label(&role.responsibility, 240))
        {
            return invalid("agent role responsibility is invalid");
        }

        let budget = &self.budget;
        if !(1..=24).contains(&budget.max_steps)
            || !(1..=32).contains(&budget.max_model_calls)
            || !(1..=40_000).contains(&budget.max_input_tokens)
            || !(1..=16_000).contains(&budget.max_output_tokens)
            || !budget.max_cost_usd.is_finite()
            || !(0.0..=1.0).contains(&budget.max_cost_usd)
            || budget.max_cost_usd == 0.0
            || !(1..=300_000).contains(&budget.timeout_ms)
            || !(1..=2).contains(&budget.max_parallel_agents)
        {
            return invalid("agent plan budget exceeds the local safety envelope");
        }

        let expected_stops = [
            StopConditionCode::BudgetExhausted,
            StopConditionCode::PermissionDenied,
            StopConditionCode::EvidenceMissing,
            StopConditionCode::Timeout,
            StopConditionCode::HumanDecisionRequired,
        ];
        if self.stop_condition_codes.as_slice() != expected_stops
            || self.stop_conditions.len() != expected_stops.len()
            || self
                .stop_conditions
                .iter()
                .any(|condition| !valid_label(condition, 160))
        {
            return invalid("agent plan stop conditions are incomplete or invalid");
        }

        Ok(())
    }
}

fn valid_plan_id(value: &str) -> bool {
    value.starts_with("plan-")
        && (6..=96).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_label(value: &str, maximum: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= maximum
        && !value.chars().any(char::is_control)
}

fn invalid<T>(message: &str) -> Result<T, AuditError> {
    Err(AuditError::Configuration(message.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"{
      "schemaVersion":"eclipse.agent-run-plan.v1",
      "planId":"plan-audit-1",
      "executionAllowed":false,
      "roles":[
        {"id":"process-analyst","responsibility":"Read-only process map"},
        {"id":"evidence-reviewer","responsibility":"Evidence bindings only"},
        {"id":"claim-auditor","responsibility":"Qualify unsupported claims"}
      ],
      "budget":{"maxSteps":12,"maxModelCalls":10,"maxInputTokens":20000,"maxOutputTokens":8000,"maxCostUsd":1,"timeoutMs":300000,"maxParallelAgents":2},
      "permissions":{"network":"deny","filesystemWrite":"deny","externalActions":"deny","secrets":"deny"},
      "stopConditionCodes":["budget_exhausted","permission_denied","evidence_missing","timeout","human_decision_required"],
      "stopConditions":["Budget","Permission","Evidence","Timeout","Human"]
    }"#;

    #[test]
    fn accepts_the_exact_ai_hub_read_only_contract() {
        let plan = AgentRunPlan::from_json(VALID).unwrap();
        assert!(!plan.execution_allowed);
        assert_eq!(plan.budget.max_parallel_agents, 2);
    }

    #[test]
    fn rejects_permission_and_execution_escalation() {
        let execution = VALID.replace("\"executionAllowed\":false", "\"executionAllowed\":true");
        assert!(AgentRunPlan::from_json(&execution).is_err());

        let network = VALID.replace("\"network\":\"deny\"", "\"network\":\"allow\"");
        assert!(AgentRunPlan::from_json(&network).is_err());
    }

    #[test]
    fn rejects_budget_growth_and_unknown_fields() {
        let budget = VALID.replace("\"maxSteps\":12", "\"maxSteps\":25");
        assert!(AgentRunPlan::from_json(&budget).is_err());

        let unknown = VALID.replace(
            "\"executionAllowed\":false",
            "\"executionAllowed\":false,\"shell\":true",
        );
        assert!(AgentRunPlan::from_json(&unknown).is_err());
    }
}
