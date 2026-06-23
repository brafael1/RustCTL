/// A systemd unit as reported by `systemctl list-units`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unit {
    pub name: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub description: String,
}

/// Whether a unit is enabled at boot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enabled {
    Enabled,
    Disabled,
    Static,
    Masked,
    Other,
}

impl Enabled {
    pub fn label(self) -> &'static str {
        match self {
            Enabled::Enabled => "enabled",
            Enabled::Disabled => "disabled",
            Enabled::Static => "static",
            Enabled::Masked => "masked",
            Enabled::Other => "other",
        }
    }
}