#[cfg(feature = "sway")]
pub mod sway {
    use anyhow::Result;
    use swayipc_async::Connection;

    pub async fn workspaces() -> Result<Vec<String>> {
        let mut c = Connection::new().await?;
        let ws = c.get_workspaces().await?;
        Ok(ws.into_iter().map(|w| w.name).collect())
    }
}

#[cfg(feature = "hypr")]
pub mod hypr {
    use anyhow::Result;
    use hyprland::data::Workspaces;
    use hyprland::shared::HyprData;
    pub async fn workspaces() -> Result<Vec<String>> {
        let workspaces = Workspaces::get()?;
        Ok(workspaces.into_iter().map(|w| w.name).collect())
    }
}
