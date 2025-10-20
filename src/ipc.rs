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
    pub async fn workspaces() -> Result<Vec<String>> {
        Ok(Workspaces::get()
            .map(|w| w.into_iter().map(|w| w.name).collect())
            .unwrap_or_default())
    }
}
