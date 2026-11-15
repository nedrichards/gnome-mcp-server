use std::collections::HashMap;

use anyhow::Result;
use serde_json::{json, Value};
use zbus::OwnedVariant;

use crate::mcp::{ToolParams, ToolProvider};

type Properties = HashMap<String, OwnedVariant>;
type MonitorSpec = (String, String, String, String);
type Mode = (String, i32, i32, f64, f64, Vec<f64>, Properties);
type PhysicalMonitor = (MonitorSpec, Vec<Mode>, Properties);
type LogicalMonitor = (i32, i32, f64, u32, bool, Vec<MonitorSpec>, Properties);
type DisplayState = (u32, Vec<PhysicalMonitor>, Vec<LogicalMonitor>, Properties);

#[derive(Default)]
pub struct Monitors;

pub struct MonitorsParams;

impl ToolParams for MonitorsParams {
    fn input_schema() -> Value {
        json!({"type": "object", "properties": {}, "required": []})
    }

    fn extract_params(arguments: &Value) -> Result<Self> {
        anyhow::ensure!(arguments.is_object(), "Monitor arguments must be an object");
        Ok(Self)
    }
}

impl ToolProvider for Monitors {
    const NAME: &'static str = "get_monitors";
    const DESCRIPTION: &'static str = "List connected monitors, current modes and logical desktop coordinates from Mutter. Read-only; does not require GNOME Shell unsafe mode. Device serial numbers are omitted.";
    type Params = MonitorsParams;

    async fn execute_with_params(&self, _params: Self::Params) -> Result<Value> {
        Self::execute_with_result(read_monitors).await
    }
}

async fn read_monitors() -> Result<Value> {
    let connection = zbus::Connection::session().await?;
    read_from_connection(&connection).await
}

async fn read_from_connection(connection: &zbus::Connection) -> Result<Value> {
    let proxy = zbus::Proxy::new(
        connection,
        "org.gnome.Mutter.DisplayConfig",
        "/org/gnome/Mutter/DisplayConfig",
        "org.gnome.Mutter.DisplayConfig",
    )
    .await?;
    let state: DisplayState = proxy.call("GetCurrentState", &()).await?;
    Ok(describe_state(state))
}

fn describe_state((_serial, physical, logical, _properties): DisplayState) -> Value {
    let monitors: Vec<Value> = physical
        .into_iter()
        .map(|((connector, _, _, _), modes, _)| {
            let current = modes.into_iter().find_map(
                |(_id, width, height, refresh_rate, preferred_scale, _scales, props)| {
                    props
                        .get("is-current")
                        .and_then(|v| bool::try_from(v).ok())
                        .unwrap_or(false)
                        .then(|| {
                            json!({"width": width, "height": height,
                            "refresh_rate": refresh_rate, "preferred_scale": preferred_scale})
                        })
                },
            );
            json!({"connector": connector, "current_mode": current})
        })
        .collect();
    let desktops: Vec<Value> = logical
        .into_iter()
        .map(|(x, y, scale, transform, primary, specs, _)| {
            let connectors: Vec<String> = specs.into_iter().map(|spec| spec.0).collect();
            json!({"x": x, "y": y, "scale": scale, "transform": transform,
                "primary": primary, "connectors": connectors})
        })
        .collect();
    json!({"monitors": monitors, "logical_monitors": desktops})
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DisplayFixture;

    #[zbus::interface(name = "org.gnome.Mutter.DisplayConfig")]
    impl DisplayFixture {
        fn get_current_state(&self) -> DisplayState {
            (1, vec![], vec![], Properties::new())
        }
    }

    #[tokio::test]
    async fn reads_the_mutter_dbus_contract_over_an_isolated_peer() {
        let (_server, client) =
            crate::test_support::peer_pair(DisplayFixture, "/org/gnome/Mutter/DisplayConfig").await;
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            read_from_connection(&client),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(result, json!({"monitors": [], "logical_monitors": []}));
        use zbus::Type;
        assert_eq!(
            DisplayState::SIGNATURE.to_string(),
            "(ua((ssss)a(siiddada{sv})a{sv})a(iiduba(ssss)a{sv})a{sv})"
        );
    }

    #[test]
    fn reports_disabled_outputs_and_scaled_mirrors_without_device_serials() {
        let spec = |name: &str| {
            (
                name.into(),
                "Invented".into(),
                "Fixture".into(),
                "private-fixture-serial".into(),
            )
        };
        let mut current = Properties::new();
        current.insert("is-current".into(), OwnedVariant::from(true));
        let mode = (
            "fixture-mode".into(),
            2560,
            1440,
            60.0,
            2.0,
            vec![1.0, 2.0],
            current,
        );
        let state = (
            1,
            vec![
                (spec("DP-1"), vec![mode], Properties::new()),
                (spec("HDMI-1"), vec![], Properties::new()),
            ],
            vec![(
                -1280,
                0,
                2.0,
                0,
                true,
                vec![spec("DP-1"), spec("DP-2")],
                Properties::new(),
            )],
            Properties::new(),
        );
        let result = describe_state(state);
        assert_eq!(result["monitors"][0]["current_mode"]["width"], 2560);
        assert!(result["monitors"][1]["current_mode"].is_null());
        assert_eq!(result["logical_monitors"][0]["x"], -1280);
        assert_eq!(result["logical_monitors"][0]["scale"], 2.0);
        assert_eq!(
            result["logical_monitors"][0]["connectors"],
            json!(["DP-1", "DP-2"])
        );
        assert!(!result.to_string().contains("private-fixture-serial"));
    }
}
