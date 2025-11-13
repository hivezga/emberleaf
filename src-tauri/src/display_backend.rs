//! Display Backend Manager for Linux WebView Stability
//!
//! Handles Wayland/X11 detection and configuration to work around
//! WebKitGTK/GBM/DMABUF issues on various Linux compositors.

use std::env;

/// Display backend selection for Linux
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayBackend {
    /// Auto-detect based on environment (Wayland if available, else X11)
    Auto,
    /// Force native Wayland
    Wayland,
    /// Force X11/XWayland
    X11,
}

impl DisplayBackend {
    /// Parse from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "auto" => Some(DisplayBackend::Auto),
            "wayland" => Some(DisplayBackend::Wayland),
            "x11" => Some(DisplayBackend::X11),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            DisplayBackend::Auto => "auto",
            DisplayBackend::Wayland => "wayland",
            DisplayBackend::X11 => "x11",
        }
    }
}

/// Detect the appropriate display backend based on environment
pub fn detect() -> DisplayBackend {
    // Check explicit override first
    if let Ok(backend_str) = env::var("EMB_DISPLAY_BACKEND") {
        if let Some(backend) = DisplayBackend::from_str(&backend_str) {
            log::info!(
                "Display backend explicitly set: EMB_DISPLAY_BACKEND={}",
                backend_str
            );
            return backend;
        } else {
            log::warn!(
                "Invalid EMB_DISPLAY_BACKEND='{}', using auto-detection",
                backend_str
            );
        }
    }

    // Auto-detect: prefer Wayland if display is available
    if env::var("WAYLAND_DISPLAY").is_ok() {
        log::info!("WAYLAND_DISPLAY detected, using Wayland backend (auto)");
        DisplayBackend::Wayland
    } else {
        log::info!("No WAYLAND_DISPLAY, using X11 backend (auto)");
        DisplayBackend::X11
    }
}

/// Apply environment variables for the selected backend
pub fn apply_env(backend: DisplayBackend) {
    let effective_backend = match backend {
        DisplayBackend::Auto => detect(),
        other => other,
    };

    match effective_backend {
        DisplayBackend::Wayland => {
            log::info!("=== Configuring Wayland Display Backend ===");

            // Set Wayland backend for winit and GDK
            env::set_var("WINIT_UNIX_BACKEND", "wayland");
            env::set_var("GDK_BACKEND", "wayland");

            // WebKit stability flags for Wayland
            // DMABUF renderer causes GBM buffer errors on many systems
            env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

            // Compositing mode can crash with certain Wayland compositors
            env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

            // Optional: Single web process (can improve stability at cost of isolation)
            // Uncomment if experiencing crashes:
            // env::set_var("WEBKIT_USE_SINGLE_WEB_PROCESS", "1");

            log::info!("  WINIT_UNIX_BACKEND=wayland");
            log::info!("  GDK_BACKEND=wayland");
            log::info!("  WEBKIT_DISABLE_DMABUF_RENDERER=1 (GBM workaround)");
            log::info!("  WEBKIT_DISABLE_COMPOSITING_MODE=1 (stability)");
        }
        DisplayBackend::X11 => {
            log::info!("=== Configuring X11 Display Backend ===");

            // Set X11 backend for winit and GDK
            env::set_var("WINIT_UNIX_BACKEND", "x11");
            env::set_var("GDK_BACKEND", "x11");

            // X11 generally doesn't need the WebKit workarounds
            // But we can keep DMABUF disabled if running under XWayland
            if env::var("WAYLAND_DISPLAY").is_ok() {
                log::info!("  Running X11 via XWayland (WAYLAND_DISPLAY present)");
                env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
                log::info!("  WEBKIT_DISABLE_DMABUF_RENDERER=1 (XWayland safety)");
            } else {
                // Pure X11, allow hardware acceleration
                env::remove_var("WEBKIT_DISABLE_DMABUF_RENDERER");
                env::remove_var("WEBKIT_DISABLE_COMPOSITING_MODE");
            }

            log::info!("  WINIT_UNIX_BACKEND=x11");
            log::info!("  GDK_BACKEND=x11");
        }
        DisplayBackend::Auto => {
            // Should not reach here (detect() resolved it)
            unreachable!("Auto backend should be resolved before apply_env")
        }
    }

    log::info!("========================================");
}

/// Check if we're running on Linux
pub fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

/// Check if we're attempting to use Wayland
pub fn is_wayland_attempt() -> bool {
    env::var("GDK_BACKEND")
        .map(|v| v == "wayland")
        .unwrap_or(false)
        || env::var("WINIT_UNIX_BACKEND")
            .map(|v| v == "wayland")
            .unwrap_or(false)
}

/// Probe for recommended Linux dependencies (non-blocking)
#[cfg(target_os = "linux")]
pub fn check_linux_dependencies() {
    use std::process::Command;

    log::info!("Probing for recommended Linux packages...");

    let packages = [
        ("webkit2gtk-4.1", "WebKitGTK"),
        ("gtk3", "GTK3"),
        ("xdg-desktop-portal", "XDG Desktop Portal"),
        ("pipewire", "PipeWire"),
    ];

    for (pkg, name) in &packages {
        match Command::new("pkg-config").arg("--exists").arg(pkg).status() {
            Ok(status) if status.success() => {
                log::debug!("  ✓ {} found", name);
            }
            _ => {
                log::warn!(
                    "  ⚠ {} not found (package: {}). Some features may be limited.",
                    name,
                    pkg
                );
            }
        }
    }

    // Check for Wayland portal implementations (not in pkg-config)
    let portals = [
        "xdg-desktop-portal-wlr",
        "xdg-desktop-portal-kde",
        "xdg-desktop-portal-gnome",
    ];

    let mut found_portal = false;
    for portal in &portals {
        if Command::new("which")
            .arg(portal)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            log::debug!("  ✓ Wayland portal: {}", portal);
            found_portal = true;
            break;
        }
    }

    if !found_portal && env::var("WAYLAND_DISPLAY").is_ok() {
        log::warn!(
            "  ⚠ No Wayland desktop portal found. Install xdg-desktop-portal-wlr (wlroots) or your compositor's variant."
        );
    }
}

#[cfg(not(target_os = "linux"))]
pub fn check_linux_dependencies() {
    // No-op on non-Linux
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_parsing() {
        assert_eq!(DisplayBackend::from_str("auto"), Some(DisplayBackend::Auto));
        assert_eq!(
            DisplayBackend::from_str("WAYLAND"),
            Some(DisplayBackend::Wayland)
        );
        assert_eq!(DisplayBackend::from_str("X11"), Some(DisplayBackend::X11));
        assert_eq!(DisplayBackend::from_str("invalid"), None);
    }

    #[test]
    fn test_backend_as_str() {
        assert_eq!(DisplayBackend::Auto.as_str(), "auto");
        assert_eq!(DisplayBackend::Wayland.as_str(), "wayland");
        assert_eq!(DisplayBackend::X11.as_str(), "x11");
    }
}
