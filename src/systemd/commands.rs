use std::process::Command;

use anyhow::{Context, Result, anyhow};

use super::types::{Enabled, Unit};

fn run(cmd: &mut Command) -> Result<String> {
    let output = cmd.output().context("failed to spawn systemctl")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("systemctl failed: {}", stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// List units, optionally filtered by type (e.g. "service", "socket", "timer").
pub fn list_units(unit_type: Option<&str>) -> Result<Vec<Unit>> {
    let mut cmd = Command::new("systemctl");
    cmd.args(["list-units", "--all", "--no-pager", "--no-legend", "--plain"]);
    if let Some(t) = unit_type {
        cmd.args(["--type", t]);
    }
    let out = run(&mut cmd)?;

    parse_list_units(&out)
}

fn parse_list_units(output: &str) -> Result<Vec<Unit>> {
    let mut units = Vec::new();
    for line in output.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        // `split_whitespace` collapses the multiple spaces that `--plain` uses
        // to align columns, so the first 4 tokens are name/load/active/sub and
        // the rest of the line (which may contain spaces) is the description.
        let mut it = line.split_whitespace();
        let name = match it.next() {
            Some(n) => n,
            None => continue,
        };
        let load_state = match it.next() {
            Some(s) => s,
            None => continue,
        };
        let active_state = match it.next() {
            Some(s) => s,
            None => continue,
        };
        let sub_state = match it.next() {
            Some(s) => s,
            None => continue,
        };
        let description: String = it.collect::<Vec<_>>().join(" ");
        units.push(Unit {
            name: name.to_string(),
            load_state: load_state.to_string(),
            active_state: active_state.to_string(),
            sub_state: sub_state.to_string(),
            description,
        });
    }
    Ok(units)
}

/// Get the enabled status of a unit (`systemctl is-enabled`).
pub fn is_enabled(name: &str) -> Enabled {
    let out = Command::new("systemctl")
        .args(["is-enabled", name])
        .output();
    match out {
        Ok(o) if o.status.success() => Enabled::Enabled,
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            match s.as_str() {
                "disabled" => Enabled::Disabled,
                "static" => Enabled::Static,
                "masked" => Enabled::Masked,
                _ => Enabled::Other,
            }
        }
        Err(_) => Enabled::Other,
    }
}

/// Detailed status output for a unit.
pub fn status(name: &str) -> Result<String> {
    let out = Command::new("systemctl")
        .args(["status", name, "--no-pager", "--lines=0"])
        .output()
        .context("failed to spawn systemctl status")?;
    // `systemctl status` returns a non-zero exit code when the unit is inactive,
    // but the stdout is still useful. Fall back to stdout first.
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    if !stdout.is_empty() {
        return Ok(stdout);
    }
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    if stderr.is_empty() {
        Ok("(no status available)".to_string())
    } else {
        Ok(stderr)
    }
}

/// Run a simple systemctl verb on a unit.
pub fn systemctl(verb: &str, name: &str) -> Result<String> {
    let out = Command::new("systemctl")
        .args([verb, name])
        .output()
        .with_context(|| format!("failed to spawn systemctl {verb}"))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        Err(anyhow!("systemctl {verb} {name}: {}", stderr.trim()))
    } else {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

pub fn start(name: &str) -> Result<String> {
    systemctl("start", name)
}
pub fn stop(name: &str) -> Result<String> {
    systemctl("stop", name)
}
pub fn restart(name: &str) -> Result<String> {
    systemctl("restart", name)
}
pub fn enable(name: &str) -> Result<String> {
    systemctl("enable", name)
}
pub fn disable(name: &str) -> Result<String> {
    systemctl("disable", name)
}
pub fn reload(name: &str) -> Result<String> {
    systemctl("reload", name)
}

#[cfg(test)]
mod tests {
    use super::parse_list_units;

    #[test]
    fn parses_a_well_formed_line() {
        let out = "sshd.service    loaded active running OpenSSH Daemon\n";
        let units = parse_list_units(out).unwrap();
        assert_eq!(units.len(), 1);
        let u = &units[0];
        assert_eq!(u.name, "sshd.service");
        assert_eq!(u.load_state, "loaded");
        assert_eq!(u.active_state, "active");
        assert_eq!(u.sub_state, "running");
        assert_eq!(u.description, "OpenSSH Daemon");
    }

    #[test]
    fn description_with_spaces_is_preserved() {
        let out = "foo.service loaded active running A long description here\n";
        let units = parse_list_units(out).unwrap();
        assert_eq!(units[0].description, "A long description here");
    }
}