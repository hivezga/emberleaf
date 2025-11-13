/**
 * Exponential Moving Average (EMA) Smoother
 *
 * Smooths noisy input signals using the formula:
 * smoothed[n] = alpha * raw[n] + (1 - alpha) * smoothed[n-1]
 *
 * Higher alpha (closer to 1.0) = more responsive, less smooth
 * Lower alpha (closer to 0.0) = less responsive, more smooth
 */

export class EMASmoother {
  private alpha: number;
  private previousValue: number | null = null;

  /**
   * Creates a new EMA smoother
   * @param alpha Smoothing factor (0.0 to 1.0), typically 0.3
   */
  constructor(alpha: number = 0.3) {
    if (alpha < 0 || alpha > 1) {
      throw new Error("Alpha must be between 0 and 1");
    }
    this.alpha = alpha;
  }

  /**
   * Apply EMA smoothing to a new raw value
   * @param rawValue The new unsmoothed input value
   * @returns The smoothed output value
   */
  smooth(rawValue: number): number {
    if (this.previousValue === null) {
      // First value: no smoothing
      this.previousValue = rawValue;
      return rawValue;
    }

    // EMA formula: smoothed = alpha * raw + (1 - alpha) * previous
    const smoothed = this.alpha * rawValue + (1 - this.alpha) * this.previousValue;
    this.previousValue = smoothed;
    return smoothed;
  }

  /**
   * Reset the smoother state
   */
  reset(): void {
    this.previousValue = null;
  }

  /**
   * Get the current smoothed value (without processing a new input)
   */
  getCurrentValue(): number | null {
    return this.previousValue;
  }
}
