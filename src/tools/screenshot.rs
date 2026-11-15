use anyhow::Result;
use ashpd::desktop::screenshot::Screenshot as ScreenshotPortal;

use crate::{mcp::ToolProvider, tool_params};

#[derive(Default)]
pub struct Screenshot;

tool_params! {
    ScreenshotParams,
    optional(interactive: bool, "Show interactive screenshot dialog for area selection"),
}

impl ToolProvider for Screenshot {
    const NAME: &'static str = "take_screenshot";
    const DESCRIPTION: &'static str =
        "Take a screenshot using the desktop portal, falling back to GNOME Shell";
    type Params = ScreenshotParams;

    async fn execute_with_params(&self, params: Self::Params) -> Result<serde_json::Value> {
        let config = crate::config::CONFIG.get_screenshot_config();

        let interactive = params.interactive.unwrap_or(config.interactive);

        Self::execute_with_result(|| take_screenshot_portal(interactive)).await
    }
}

async fn take_screenshot_portal(interactive: bool) -> Result<String> {
    let portal_result: std::result::Result<String, ashpd::Error> = async {
        let response = ScreenshotPortal::request()
            .interactive(interactive)
            .send()
            .await?
            .response()?;
        let uri = response.uri();
        Ok(if interactive {
            format!("Interactive screenshot completed. File saved to: {uri}")
        } else {
            format!("Screenshot taken. File saved to: {uri}")
        })
    }
    .await;

    match portal_result {
        Ok(result) => Ok(result),
        Err(portal_error) if !fallback_allowed(&portal_error) => Err(portal_error.into()),
        Err(portal_error) => {
            let shell_result = if interactive {
                take_interactive_screenshot_shell().await
            } else {
                take_screenshot_shell().await
            };

            shell_result.map_err(|shell_error| {
                anyhow::anyhow!(
                    "Portal screenshot failed: {portal_error}; GNOME Shell fallback failed: {shell_error}"
                )
            })
        }
    }
}

async fn take_screenshot_shell() -> Result<String> {
    let connection = zbus::Connection::session().await?;
    let proxy = zbus::Proxy::new(
        &connection,
        "org.gnome.Shell.Screenshot",
        "/org/gnome/Shell/Screenshot",
        "org.gnome.Shell.Screenshot",
    )
    .await?;

    let filename = format!(
        "/tmp/gnome-mcp-screenshot-{}.png",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
    );

    let response = proxy
        .call_method("Screenshot", &(false, false, filename.as_str()))
        .await?;
    let (success, filename_used): (bool, String) = response.body().deserialize()?;

    if success {
        Ok(format!("Screenshot taken. File saved to: {filename_used}"))
    } else {
        Err(anyhow::anyhow!("GNOME Shell screenshot failed"))
    }
}

async fn take_interactive_screenshot_shell() -> Result<String> {
    let connection = zbus::Connection::session().await?;
    let proxy = zbus::Proxy::new(
        &connection,
        "org.gnome.Shell.Screenshot",
        "/org/gnome/Shell/Screenshot",
        "org.gnome.Shell.Screenshot",
    )
    .await?;

    let response = proxy.call_method("InteractiveScreenshot", &()).await?;
    let (success, uri): (bool, String) = response.body().deserialize()?;

    if success {
        Ok(format!(
            "Interactive screenshot completed. File saved to: {uri}"
        ))
    } else {
        Err(anyhow::anyhow!("GNOME Shell interactive screenshot failed"))
    }
}

// A negative portal response is final; never capture through another backend
// after the user cancels or permission is refused.
fn fallback_allowed(error: &ashpd::Error) -> bool {
    match error {
        ashpd::Error::Response(_) => false,
        ashpd::Error::Portal(
            ashpd::PortalError::Cancelled(_) | ashpd::PortalError::NotAllowed(_),
        ) => false,
        ashpd::Error::Zbus(zbus::Error::MethodError(name, _, _)) => !matches!(
            name.as_str(),
            "org.freedesktop.DBus.Error.AccessDenied"
                | "org.freedesktop.portal.Error.NotAllowed"
                | "org.freedesktop.portal.Error.Cancelled"
        ),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_and_denial_do_not_fall_back_to_shell_capture() {
        assert!(!fallback_allowed(&ashpd::Error::Response(
            ashpd::desktop::ResponseError::Cancelled
        )));
        assert!(!fallback_allowed(&ashpd::Error::Portal(
            ashpd::PortalError::NotAllowed("fixture denial".into())
        )));
        assert!(fallback_allowed(&ashpd::Error::PortalNotFound(
            "org.freedesktop.portal.Screenshot".try_into().unwrap()
        )));
    }
}
