use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PingParams {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum PaneAgentState {
    Idle,
    Working,
    Blocked,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PaneReportAgentParams {
    pub pane_id: String,
    pub source: String,
    pub agent: String,
    pub state: PaneAgentState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_session_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PaneReportAgentSessionParams {
    pub pane_id: String,
    pub source: String,
    pub agent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_session_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_start_source: Option<String>,
}

/// Newline-JSON request surface used by integration hooks.
///
/// This is not Herdr's bincode client/server protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ReportRequest {
    pub id: String,
    #[serde(flatten)]
    pub method: ReportMethod,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "method", content = "params")]
pub enum ReportMethod {
    #[serde(rename = "ping")]
    Ping(PingParams),
    #[serde(rename = "pane.report_agent")]
    PaneReportAgent(PaneReportAgentParams),
    #[serde(rename = "pane.report_agent_session")]
    PaneReportAgentSession(PaneReportAgentSessionParams),
}

impl ReportRequest {
    pub fn from_newline_json(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line.trim())
    }

    pub fn to_newline_json(&self) -> Result<String, serde_json::Error> {
        let mut encoded = serde_json::to_string(self)?;
        if !encoded.ends_with('\n') {
            encoded.push('\n');
        }
        Ok(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_agent_round_trips_newline_json() {
        let request = ReportRequest {
            id: "herdr:pi:1".into(),
            method: ReportMethod::PaneReportAgent(PaneReportAgentParams {
                pane_id: "%1".into(),
                source: "herdr:pi".into(),
                agent: "pi".into(),
                state: PaneAgentState::Working,
                message: None,
                seq: Some(3),
                agent_session_id: Some("sess".into()),
                agent_session_path: None,
            }),
        };
        let encoded = request.to_newline_json().unwrap();
        let decoded = ReportRequest::from_newline_json(&encoded).unwrap();
        assert_eq!(decoded, request);
        assert!(encoded.contains("\"method\":\"pane.report_agent\""));
    }

    #[test]
    fn report_agent_session_and_ping_decode() {
        let session = ReportRequest::from_newline_json(
            r#"{"id":"1","method":"pane.report_agent_session","params":{"pane_id":"%2","source":"herdr:omp","agent":"omp"}}"#,
        )
        .unwrap();
        assert!(matches!(
            session.method,
            ReportMethod::PaneReportAgentSession(_)
        ));
        let ping =
            ReportRequest::from_newline_json(r#"{"id":"2","method":"ping","params":{}}"#).unwrap();
        assert!(matches!(ping.method, ReportMethod::Ping(_)));
    }
}
