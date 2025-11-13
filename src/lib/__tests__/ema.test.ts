/**
 * Unit tests for EMA Smoother
 *
 * Run with: npm test (requires vitest setup)
 */

import { describe, it, expect, beforeEach } from "vitest";
import { EMASmoother } from "../ema";

describe("EMASmoother", () => {
  let smoother: EMASmoother;

  beforeEach(() => {
    smoother = new EMASmoother(0.3);
  });

  it("should return the first value unchanged", () => {
    const result = smoother.smooth(0.5);
    expect(result).toBe(0.5);
  });

  it("should apply EMA formula correctly with alpha=0.3", () => {
    // Test sequence: [0, 1, 0.5, 0.25]
    // Expected outputs:
    // smooth(0) = 0 (first value)
    // smooth(1) = 0.3 * 1 + 0.7 * 0 = 0.3
    // smooth(0.5) = 0.3 * 0.5 + 0.7 * 0.3 = 0.15 + 0.21 = 0.36
    // smooth(0.25) = 0.3 * 0.25 + 0.7 * 0.36 = 0.075 + 0.252 = 0.327

    const result1 = smoother.smooth(0);
    expect(result1).toBeCloseTo(0, 5);

    const result2 = smoother.smooth(1);
    expect(result2).toBeCloseTo(0.3, 5);

    const result3 = smoother.smooth(0.5);
    expect(result3).toBeCloseTo(0.36, 5);

    const result4 = smoother.smooth(0.25);
    expect(result4).toBeCloseTo(0.327, 5);
  });

  it("should smooth out noise in a noisy signal", () => {
    // Simulate a signal with noise
    const noisySignal = [0.5, 0.6, 0.4, 0.55, 0.45, 0.5];
    const smoothed: number[] = [];

    noisySignal.forEach((value) => {
      smoothed.push(smoother.smooth(value));
    });

    // Smoothed values should vary less than raw values
    const rawVariance = calculateVariance(noisySignal);
    const smoothedVariance = calculateVariance(smoothed);

    expect(smoothedVariance).toBeLessThan(rawVariance);
  });

  it("should reset state when reset() is called", () => {
    smoother.smooth(0.5);
    smoother.smooth(0.8);
    expect(smoother.getCurrentValue()).toBeCloseTo(0.59, 5); // 0.3 * 0.8 + 0.7 * 0.5

    smoother.reset();
    expect(smoother.getCurrentValue()).toBeNull();

    // Next value should be treated as first value
    const result = smoother.smooth(1.0);
    expect(result).toBe(1.0);
  });

  it("should throw error for invalid alpha values", () => {
    expect(() => new EMASmoother(-0.1)).toThrow("Alpha must be between 0 and 1");
    expect(() => new EMASmoother(1.5)).toThrow("Alpha must be between 0 and 1");
  });

  it("should handle alpha=0 (no updates)", () => {
    const zeroAlpha = new EMASmoother(0);
    zeroAlpha.smooth(1.0);
    const result = zeroAlpha.smooth(0.5);
    expect(result).toBe(1.0); // Should stay at first value
  });

  it("should handle alpha=1 (no smoothing)", () => {
    const oneAlpha = new EMASmoother(1.0);
    oneAlpha.smooth(0.5);
    const result = oneAlpha.smooth(0.8);
    expect(result).toBe(0.8); // Should just return raw value
  });

  it("should converge to steady state", () => {
    // Feed constant value, should converge to that value
    const target = 0.7;
    let result = 0;

    for (let i = 0; i < 50; i++) {
      result = smoother.smooth(target);
    }

    expect(result).toBeCloseTo(target, 3);
  });
});

// Helper function to calculate variance
function calculateVariance(values: number[]): number {
  const mean = values.reduce((sum, val) => sum + val, 0) / values.length;
  const squaredDiffs = values.map((val) => Math.pow(val - mean, 2));
  return squaredDiffs.reduce((sum, val) => sum + val, 0) / values.length;
}
