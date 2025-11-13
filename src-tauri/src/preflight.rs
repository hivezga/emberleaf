/**
 * Preflight checks for Linux dependencies and permissions
 *
 * Verifies audio stack, webkit, portals, and mic access before onboarding.
 * Emits events for UI feedback and returns structured report.
 */
use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::{AppHandle, Emitter};

/// Status of an individual preflight check
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

/// Individual preflight check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightItem {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub fix_hint: Option<String>,
}

/// Complete preflight report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightReport {
    pub items: Vec<PreflightItem>,
    pub overall: CheckStatus,
    pub can_proceed: bool,
}

impl PreflightReport {
    /// Determine overall status from individual checks
    fn compute_overall(items: &[PreflightItem]) -> CheckStatus {
        if items.iter().any(|i| i.status == CheckStatus::Fail) {
            CheckStatus::Fail
        } else if items.iter().any(|i| i.status == CheckStatus::Warn) {
            CheckStatus::Warn
        } else {
            CheckStatus::Pass
        }
    }

    /// Can app proceed despite warnings?
    fn can_proceed(items: &[PreflightItem]) -> bool {
        !items.iter().any(|i| i.status == CheckStatus::Fail)
    }
}

/// Run all preflight checks and emit events
pub fn run_preflight(app: &AppHandle) -> PreflightReport {
    log::info!("Starting preflight checks...");

    let _ = app.emit("preflight:started", ());

    let mut items = Vec::new();

    // Check 1: Audio stack (PipeWire or PulseAudio)
    let audio_check = check_audio_stack();
    let _ = app.emit("preflight:item", &audio_check);
    items.push(audio_check);

    // Check 2: WebKit2GTK minimum version
    let webkit_check = check_webkit();
    let _ = app.emit("preflight:item", &webkit_check);
    items.push(webkit_check);

    // Check 3: XDG Desktop Portal
    let portal_check = check_portal();
    let _ = app.emit("preflight:item", &portal_check);
    items.push(portal_check);

    // Check 4: Microphone permissions/access
    let mic_check = check_mic_access();
    let _ = app.emit("preflight:item", &mic_check);
    items.push(mic_check);

    let overall = PreflightReport::compute_overall(&items);
    let can_proceed = PreflightReport::can_proceed(&items);

    let report = PreflightReport {
        items,
        overall: overall.clone(),
        can_proceed,
    };

    let _ = app.emit("preflight:done", &report);

    log::info!("Preflight complete: {:?}", overall);
    report
}

/// Check for PipeWire or PulseAudio
fn check_audio_stack() -> PreflightItem {
    // Try PipeWire first
    if Command::new("pw-cli").arg("info").output().is_ok() {
        return PreflightItem {
            name: "audio_stack".to_string(),
            status: CheckStatus::Pass,
            message: "PipeWire detected".to_string(),
            fix_hint: None,
        };
    }

    // Fallback to PulseAudio check
    if Command::new("pactl").arg("info").output().is_ok() {
        return PreflightItem {
            name: "audio_stack".to_string(),
            status: CheckStatus::Pass,
            message: "PulseAudio detected".to_string(),
            fix_hint: None,
        };
    }

    // Neither found
    PreflightItem {
        name: "audio_stack".to_string(),
        status: CheckStatus::Fail,
        message: "No audio server detected (PipeWire or PulseAudio required)".to_string(),
        fix_hint: Some(
            "Install PipeWire or PulseAudio:\n\
             • Arch: sudo pacman -S pipewire pipewire-pulse\n\
             • Ubuntu/Debian: sudo apt install pipewire pipewire-pulse\n\
             • Fedora: sudo dnf install pipewire pipewire-pulseaudio"
                .to_string(),
        ),
    }
}

/// Check WebKit2GTK version
fn check_webkit() -> PreflightItem {
    // Try to detect webkit2gtk-4.1 or 4.0
    let check_41 = Command::new("pkg-config")
        .args(&["--modversion", "webkit2gtk-4.1"])
        .output();

    let check_40 = Command::new("pkg-config")
        .args(&["--modversion", "webkit2gtk-4.0"])
        .output();

    if check_41.is_ok() || check_40.is_ok() {
        PreflightItem {
            name: "webkit".to_string(),
            status: CheckStatus::Pass,
            message: "WebKit2GTK found".to_string(),
            fix_hint: None,
        }
    } else {
        PreflightItem {
            name: "webkit".to_string(),
            status: CheckStatus::Fail,
            message: "WebKit2GTK not found".to_string(),
            fix_hint: Some(
                "Install WebKit2GTK:\n\
                 • Arch: sudo pacman -S webkit2gtk\n\
                 • Ubuntu/Debian: sudo apt install libwebkit2gtk-4.0-dev\n\
                 • Fedora: sudo dnf install webkit2gtk3"
                    .to_string(),
            ),
        }
    }
}

/// Check for XDG Desktop Portal
fn check_portal() -> PreflightItem {
    // Check if xdg-desktop-portal is running
    let portal_running = Command::new("pgrep")
        .arg("xdg-desktop-portal")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if portal_running {
        PreflightItem {
            name: "portal".to_string(),
            status: CheckStatus::Pass,
            message: "XDG Desktop Portal running".to_string(),
            fix_hint: None,
        }
    } else {
        // Check if it's installed but not running
        let portal_exists = Command::new("which")
            .arg("xdg-desktop-portal")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if portal_exists {
            PreflightItem {
                name: "portal".to_string(),
                status: CheckStatus::Warn,
                message: "XDG Desktop Portal installed but not running".to_string(),
                fix_hint: Some(
                    "Start the portal service:\n\
                     • systemctl --user start xdg-desktop-portal\n\
                     Or reboot to auto-start"
                        .to_string(),
                ),
            }
        } else {
            PreflightItem {
                name: "portal".to_string(),
                status: CheckStatus::Warn,
                message: "XDG Desktop Portal not found (may affect Wayland)".to_string(),
                fix_hint: Some(
                    "Install portal for better Wayland support:\n\
                     • Arch: sudo pacman -S xdg-desktop-portal xdg-desktop-portal-gtk\n\
                     • Ubuntu/Debian: sudo apt install xdg-desktop-portal xdg-desktop-portal-gtk\n\
                     • Fedora: sudo dnf install xdg-desktop-portal xdg-desktop-portal-gtk"
                        .to_string(),
                ),
            }
        }
    }
}

/// Check microphone access (basic probe)
fn check_mic_access() -> PreflightItem {
    // Use CPAL to try listing input devices
    use cpal::traits::HostTrait;

    match cpal::default_host().input_devices() {
        Ok(mut devices) => {
            if devices.next().is_some() {
                PreflightItem {
                    name: "mic_access".to_string(),
                    status: CheckStatus::Pass,
                    message: "Microphone devices found".to_string(),
                    fix_hint: None,
                }
            } else {
                PreflightItem {
                    name: "mic_access".to_string(),
                    status: CheckStatus::Warn,
                    message: "No microphone devices detected".to_string(),
                    fix_hint: Some("Connect a microphone or check audio settings".to_string()),
                }
            }
        }
        Err(e) => PreflightItem {
            name: "mic_access".to_string(),
            status: CheckStatus::Fail,
            message: format!("Cannot access audio devices: {}", e),
            fix_hint: Some(
                "Check permissions and audio configuration:\n\
                 • Ensure user is in 'audio' group: sudo usermod -aG audio $USER\n\
                 • Verify audio server is running (PipeWire/PulseAudio)"
                    .to_string(),
            ),
        },
    }
}

/// Tauri command: Run preflight checks
#[tauri::command]
pub async fn run_preflight_checks(app: AppHandle) -> Result<PreflightReport, String> {
    Ok(run_preflight(&app))
}
