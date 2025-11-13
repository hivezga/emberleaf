import { describe, it, expect } from "vitest";
import { format, translations } from "./translations";

describe("i18n translations", () => {
  describe("format function", () => {
    it("should replace single placeholder", () => {
      const result = format("Hello {name}", { name: "World" });
      expect(result).toBe("Hello World");
    });

    it("should replace multiple placeholders", () => {
      const result = format("Microphone: {device} ({status})", {
        device: "USB Mic",
        status: "Good",
      });
      expect(result).toBe("Microphone: USB Mic (Good)");
    });

    it("should handle missing placeholders gracefully", () => {
      const result = format("Hello {name}", {});
      expect(result).toBe("Hello {name}");
    });

    it("should work with real translation strings", () => {
      const result = format(translations.en.micDetected, { device: "Built-in Microphone" });
      expect(result).toContain("Built-in Microphone");
      expect(result).toContain("Good");
    });
  });

  describe("translation keys", () => {
    it("should have matching keys in English and Spanish", () => {
      const enKeys = Object.keys(translations.en);
      const esKeys = Object.keys(translations.es);
      expect(enKeys.sort()).toEqual(esKeys.sort());
    });

    it("should have preflight translations", () => {
      expect(translations.en.preflight).toBeDefined();
      expect(translations.es.preflight).toBeDefined();
      expect(translations.en.preflight.title).toBeTruthy();
      expect(translations.es.preflight.title).toBeTruthy();
    });

    it("should have all preflight status strings", () => {
      expect(translations.en.preflight.statusReady).toBeTruthy();
      expect(translations.en.preflight.statusWarning).toBeTruthy();
      expect(translations.en.preflight.statusError).toBeTruthy();
      expect(translations.es.preflight.statusReady).toBeTruthy();
      expect(translations.es.preflight.statusWarning).toBeTruthy();
      expect(translations.es.preflight.statusError).toBeTruthy();
    });
  });
});
