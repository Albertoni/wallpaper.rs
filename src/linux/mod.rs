use download_image;
use enquote::enquote;
use std::env;
use std::process::Command;
use Result;

/// Returns the wallpaper of the current desktop.
pub fn get() -> Result<String> {
    let desktop = env::var("XDG_CURRENT_DESKTOP")?;

    if is_gnome_compliant(&desktop) {
        return parse_dconf(
            "gsettings",
            &["get", "org.gnome.desktop.background", "picture-uri"],
        );
    }

    match desktop.as_str() {
        "KDE" => Err("TODO".into()),
        "X-Cinnamon" => parse_dconf(
            "dconf",
            &["read", "/org/cinnamon/desktop/background/picture-uri"],
        ),
        "MATE" => parse_dconf(
            "dconf",
            &["read", "/org/mate/desktop/background/picture-filename"],
        ),
        "XFCE" => Err("TODO".into()),
        "LXDE" => Err("TODO".into()),
        "Deepin" => parse_dconf(
            "dconf",
            &[
                "read",
                "/com/deepin/wrap/gnome/desktop/background/picture-uri",
            ],
        ),
        _ => Err("unsupported desktop".into()),
    }
}

/// Sets the wallpaper for the current desktop from a file path.
pub fn set_from_path(path: &str) -> Result<()> {
    let desktop = env::var("XDG_CURRENT_DESKTOP")?;

    if is_gnome_compliant(&desktop) {
        let uri = enquote('"', &format!("file://{}", path));
        Command::new("gsettings")
            .args(&["set", "org.gnome.desktop.background", "picture-uri", &uri])
            .output()?;
        return Ok(());
    }

    match desktop.as_str() {
        "KDE" => Err("TODO".into()),
        "X-Cinnamon" => run(
            "dconf",
            &[
                "write",
                "/org/cinnamon/desktop/background/picture-uri",
                &enquote('"', &format!("file://{}", path)),
            ],
        ),
        "MATE" => run(
            "dconf",
            &[
                "write",
                "/org/mate/desktop/background/picture-filename",
                &enquote('"', &path),
            ],
        ),
        "XFCE" => run(
            "xfconf-query",
            &[
                "-c",
                "xfce4-desktop",
                "-p",
                "/backdrop/screen0/monitor0/workspace0/last-image",
                "-s",
                &path,
            ],
        ),
        "LXDE" => run("pcmanfm", &["-w", &path]),
        "Deepin" => run(
            "dconf",
            &[
                "write",
                "/com/deepin/wrap/gnome/desktop/background/picture-uri",
                &enquote('"', &format!("file://{}", path)),
            ],
        ),
        "i3" => run("feh", &["--bg-fill", &path]),
        _ => Err("unsupported desktop".into()),
    }
}

/// Sets the wallpaper for the current desktop from a URL.
pub fn set_from_url(url: &str) -> Result<()> {
    let desktop = env::var("XDG_CURRENT_DESKTOP")?;

    match desktop.as_str() {
        // only some GNOME-based desktops support urls for picture-uri
        "GNOME" | "ubuntu:GNOME" => run(
            "gsettings",
            &[
                "set",
                "org.gnome.desktop.background",
                "picture-uri",
                &enquote('"', url),
            ],
        ),
        "i3" => run("feh", &["--bg-fill", url]),
        _ => {
            let path = download_image(&url.parse()?)?;
            set_from_path(&path)
        }
    }
}

#[inline]
fn is_gnome_compliant(desktop: &str) -> bool {
    desktop.contains("GNOME") || desktop == "Unity" || desktop == "Pantheon"
}

fn parse_dconf(command: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(command).args(args).output()?;
    if !output.status.success() {
        return Err(format!(
            "{} exited with status code {}",
            command,
            output.status.code().unwrap_or(-1),
        ).into());
    }

    let mut stdout = String::from_utf8(output.stdout)?.trim().to_owned();

    // unquotes single quotes
    stdout.remove(0);
    stdout.pop();
    stdout = stdout.replace("\\'", "'");

    // removes file protocol
    if stdout.starts_with("file://") {
        stdout = stdout.split_at(7).1.into();
    }

    Ok(stdout)
}

fn run(command: &str, args: &[&str]) -> Result<()> {
    let output = Command::new(command).args(args).output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{} exited with status code {}",
            command,
            output.status.code().unwrap_or(-1)
        ).into())
    }
}