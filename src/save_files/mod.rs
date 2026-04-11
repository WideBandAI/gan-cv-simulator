pub mod cv_curves;
pub mod interface_states;
pub mod potential_profile;

pub(crate) fn validate_save_dir(save_dir: &str) -> anyhow::Result<()> {
    if std::path::Path::new(save_dir)
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        anyhow::bail!("Invalid save directory: contains path traversal components.");
    }
    Ok(())
}
