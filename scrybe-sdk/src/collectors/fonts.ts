/**
 * Font detection collector
 * 
 * Detects installed fonts using canvas text measurement.
 * Uses a baseline font and measures width differences.
 */

import type { FontFingerprint } from '../types';
import { sha256 } from '../utils/hash';

export class FontCollector {
  // Common fonts to test
  private readonly testFonts = [
    // Windows fonts
    'Arial', 'Verdana', 'Courier New', 'Times New Roman', 'Georgia',
    'Palatino Linotype', 'Book Antiqua', 'Comic Sans MS', 'Impact',
    'Lucida Console', 'Tahoma', 'Trebuchet MS', 'Arial Black',
    
    // Mac fonts
    'Monaco', 'Menlo', 'Helvetica Neue', 'Geneva', 'Apple Chancery',
    'Baskerville', 'Big Caslon', 'Copperplate', 'Didot', 'Futura',
    'Gill Sans', 'Optima', 'Palatino',
    
    // Linux fonts
    'Liberation Sans', 'Liberation Serif', 'Liberation Mono',
    'DejaVu Sans', 'DejaVu Serif', 'DejaVu Sans Mono',
    'Ubuntu', 'Cantarell', 'Droid Sans', 'Noto Sans',
    
    // Web fonts (if installed locally)
    'Roboto', 'Open Sans', 'Lato', 'Montserrat', 'Source Sans Pro',
  ];

  private readonly baseFonts = ['monospace', 'sans-serif', 'serif'];
  private readonly testString = 'mmmmmmmmmmlli'; // Mix of wide and thin chars
  private readonly testSize = '72px';

  /**
   * Collect font fingerprint
   * 
   * @returns Promise resolving to font fingerprint
   */
  async collect(): Promise<FontFingerprint | undefined> {
    try {
      const availableFonts = this.detectFonts();
      
      if (availableFonts.length === 0) {
        return undefined;
      }

      // Sort for determinism
      availableFonts.sort();

      // Hash the font list
      const combined = availableFonts.join('|');
      const hash = await sha256(combined);

      return {
        available: availableFonts,
        hash,
      };
    } catch (error) {
      return undefined;
    }
  }

  /**
   * Detect installed fonts
   * 
   * @private
   */
  private detectFonts(): string[] {
    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d');

    if (!context) {
      return [];
    }

    // Get baseline measurements
    const baseMeasurements = new Map<string, { width: number; height: number }>();

    for (const baseFont of this.baseFonts) {
      context.font = `${this.testSize} ${baseFont}`;
      const metrics = context.measureText(this.testString);
      baseMeasurements.set(baseFont, {
        width: metrics.width,
        height: metrics.actualBoundingBoxAscent + metrics.actualBoundingBoxDescent,
      });
    }

    // Test each font
    const detected: string[] = [];

    for (const testFont of this.testFonts) {
      let hasDifference = false;

      for (const baseFont of this.baseFonts) {
        context.font = `${this.testSize} '${testFont}', ${baseFont}`;
        const metrics = context.measureText(this.testString);
        const measurement = {
          width: metrics.width,
          height: metrics.actualBoundingBoxAscent + metrics.actualBoundingBoxDescent,
        };

        const baseline = baseMeasurements.get(baseFont);
        if (!baseline) continue;

        // If measurements differ, the font is likely installed
        if (
          Math.abs(measurement.width - baseline.width) > 0.5 ||
          Math.abs(measurement.height - baseline.height) > 0.5
        ) {
          hasDifference = true;
          break;
        }
      }

      if (hasDifference) {
        detected.push(testFont);
      }
    }

    return detected;
  }
}
