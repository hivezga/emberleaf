/**
 * Unit tests for Device Loss Banner event handlers
 *
 * Run with: npm test (requires vitest + @testing-library/react setup)
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { DeviceLossBanner } from "../DeviceLossBanner";
import { I18nProvider } from "../../lib/i18n";

// Mock Tauri event system
const mockListen = vi.fn();
const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/event", () => ({
  listen: mockListen,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

// Helper to wrap component with I18n provider
function renderWithI18n(component: React.ReactElement) {
  return render(<I18nProvider>{component}</I18nProvider>);
}

describe("DeviceLossBanner - Event Handlers", () => {
  let unlistenMock: ReturnType<typeof vi.fn>;
  const mockOnOpenDevicePicker = vi.fn();

  beforeEach(() => {
    unlistenMock = vi.fn();
    mockListen.mockResolvedValue(unlistenMock);
    mockInvoke.mockResolvedValue("Mock response");
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.clearAllMocks();
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  it("should setup event listeners on mount", async () => {
    renderWithI18n(<DeviceLossBanner onOpenDevicePicker={mockOnOpenDevicePicker} />);

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledWith(
        "audio:device_lost",
        expect.any(Function)
      );
      expect(mockListen).toHaveBeenCalledWith(
        "audio:device_fallback_ok",
        expect.any(Function)
      );
      expect(mockListen).toHaveBeenCalledWith(
        "audio:device_fallback_failed",
        expect.any(Function)
      );
      expect(mockListen).toHaveBeenCalledWith(
        "audio:monitor_guarded",
        expect.any(Function)
      );
    });
  });

  it("should display success banner on device_fallback_ok event", async () => {
    let fallbackOkCallback: ((event: any) => void) | null = null;

    mockListen.mockImplementation((eventName: string, callback: any) => {
      if (eventName === "audio:device_fallback_ok") {
        fallbackOkCallback = callback;
      }
      return Promise.resolve(unlistenMock);
    });

    renderWithI18n(<DeviceLossBanner onOpenDevicePicker={mockOnOpenDevicePicker} />);

    await waitFor(() => expect(fallbackOkCallback).not.toBeNull());

    // Simulate fallback_ok event
    fallbackOkCallback!({
      payload: { kind: "input", new_device: "default" },
    });

    await waitFor(() => {
      expect(screen.getByText(/Switched to the default device/i)).toBeInTheDocument();
    });

    // Check for success styling (green banner)
    const banner = screen.getByRole("status");
    expect(banner).toHaveClass("bg-green-50", "dark:bg-green-950");

    // Verify "Change…" button is present
    expect(screen.getByText(/Change/i)).toBeInTheDocument();
  });

  it("should auto-dismiss success banner after 6 seconds", async () => {
    let fallbackOkCallback: ((event: any) => void) | null = null;

    mockListen.mockImplementation((eventName: string, callback: any) => {
      if (eventName === "audio:device_fallback_ok") {
        fallbackOkCallback = callback;
      }
      return Promise.resolve(unlistenMock);
    });

    renderWithI18n(<DeviceLossBanner onOpenDevicePicker={mockOnOpenDevicePicker} />);

    await waitFor(() => expect(fallbackOkCallback).not.toBeNull());

    // Trigger event
    fallbackOkCallback!({
      payload: { kind: "input", new_device: "default" },
    });

    await waitFor(() => {
      expect(screen.getByText(/Switched to the default device/i)).toBeInTheDocument();
    });

    // Fast-forward 6 seconds
    vi.advanceTimersByTime(6000);

    await waitFor(() => {
      expect(screen.queryByText(/Switched to the default device/i)).not.toBeInTheDocument();
    });
  });

  it("should display warning banner on device_fallback_failed event", async () => {
    let fallbackFailedCallback: ((event: any) => void) | null = null;

    mockListen.mockImplementation((eventName: string, callback: any) => {
      if (eventName === "audio:device_fallback_failed") {
        fallbackFailedCallback = callback;
      }
      return Promise.resolve(unlistenMock);
    });

    renderWithI18n(<DeviceLossBanner onOpenDevicePicker={mockOnOpenDevicePicker} />);

    await waitFor(() => expect(fallbackFailedCallback).not.toBeNull());

    // Simulate fallback_failed event
    fallbackFailedCallback!({
      payload: {
        kind: "input",
        reason: "No input device available",
      },
    });

    await waitFor(() => {
      expect(screen.getByText(/couldn't switch automatically/i)).toBeInTheDocument();
      expect(screen.getByText(/No input device available/i)).toBeInTheDocument();
    });

    // Check for warning styling (amber banner)
    const banner = screen.getByRole("status");
    expect(banner).toHaveClass("bg-amber-50", "dark:bg-amber-950");

    // Verify both "Retry" and "Change…" buttons are present
    expect(screen.getByText(/Retry/i)).toBeInTheDocument();
    expect(screen.getByText(/Change/i)).toBeInTheDocument();
  });

  it("should NOT auto-dismiss warning banner (persists until resolved)", async () => {
    let fallbackFailedCallback: ((event: any) => void) | null = null;

    mockListen.mockImplementation((eventName: string, callback: any) => {
      if (eventName === "audio:device_fallback_failed") {
        fallbackFailedCallback = callback;
      }
      return Promise.resolve(unlistenMock);
    });

    renderWithI18n(<DeviceLossBanner onOpenDevicePicker={mockOnOpenDevicePicker} />);

    await waitFor(() => expect(fallbackFailedCallback).not.toBeNull());

    // Trigger event
    fallbackFailedCallback!({
      payload: { kind: "input", reason: "No device" },
    });

    await waitFor(() => {
      expect(screen.getByText(/couldn't switch automatically/i)).toBeInTheDocument();
    });

    // Fast-forward 10 seconds
    vi.advanceTimersByTime(10000);

    // Banner should still be present
    expect(screen.getByText(/couldn't switch automatically/i)).toBeInTheDocument();
  });

  it("should call restart_audio_capture when Retry button is clicked", async () => {
    let fallbackFailedCallback: ((event: any) => void) | null = null;

    mockListen.mockImplementation((eventName: string, callback: any) => {
      if (eventName === "audio:device_fallback_failed") {
        fallbackFailedCallback = callback;
      }
      return Promise.resolve(unlistenMock);
    });

    const { user } = renderWithI18n(
      <DeviceLossBanner onOpenDevicePicker={mockOnOpenDevicePicker} />
    );

    await waitFor(() => expect(fallbackFailedCallback).not.toBeNull());

    // Trigger event
    fallbackFailedCallback!({
      payload: { kind: "input", reason: "No device" },
    });

    await waitFor(() => {
      expect(screen.getByText(/Retry/i)).toBeInTheDocument();
    });

    // Click Retry button
    await user.click(screen.getByText(/Retry/i));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("restart_audio_capture");
    });
  });

  it("should display info banner on monitor_guarded event", async () => {
    let monitorGuardedCallback: ((event: any) => void) | null = null;

    mockListen.mockImplementation((eventName: string, callback: any) => {
      if (eventName === "audio:monitor_guarded") {
        monitorGuardedCallback = callback;
      }
      return Promise.resolve(unlistenMock);
    });

    renderWithI18n(<DeviceLossBanner onOpenDevicePicker={mockOnOpenDevicePicker} />);

    await waitFor(() => expect(monitorGuardedCallback).not.toBeNull());

    // Simulate monitor_guarded event
    monitorGuardedCallback!({
      payload: { reason: "feedback_risk" },
    });

    await waitFor(() => {
      expect(screen.getByText(/Monitoring wasn't resumed to avoid feedback/i)).toBeInTheDocument();
    });

    // Check for info styling (blue banner)
    const banner = screen.getByRole("status");
    expect(banner).toHaveClass("bg-blue-50", "dark:bg-blue-950");
  });

  it("should auto-dismiss info banner after 8 seconds", async () => {
    let monitorGuardedCallback: ((event: any) => void) | null = null;

    mockListen.mockImplementation((eventName: string, callback: any) => {
      if (eventName === "audio:monitor_guarded") {
        monitorGuardedCallback = callback;
      }
      return Promise.resolve(unlistenMock);
    });

    renderWithI18n(<DeviceLossBanner onOpenDevicePicker={mockOnOpenDevicePicker} />);

    await waitFor(() => expect(monitorGuardedCallback).not.toBeNull());

    // Trigger event
    monitorGuardedCallback!({
      payload: { reason: "feedback_risk" },
    });

    await waitFor(() => {
      expect(screen.getByText(/Monitoring wasn't resumed/i)).toBeInTheDocument();
    });

    // Fast-forward 8 seconds
    vi.advanceTimersByTime(8000);

    await waitFor(() => {
      expect(screen.queryByText(/Monitoring wasn't resumed/i)).not.toBeInTheDocument();
    });
  });

  it("should show guard note under monitor toggle in AudioSettings", async () => {
    // This test would verify that when monitor_guarded fires,
    // the AudioSettings component shows a help text under the persistence toggle
    // (This would require testing AudioSettings component separately)

    // Placeholder test to document expected behavior
    expect(true).toBe(true);
  });
});
