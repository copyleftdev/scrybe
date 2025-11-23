/**
 * Canvas fingerprinting collector
 * 
 * Generates a unique fingerprint by rendering specific patterns on a canvas
 * and hashing the resulting pixel data.
 */

import type { CanvasFingerprint } from '../types';
import { sha256 } from '../utils/hash';

export class CanvasCollector {
  /**
   * Collect canvas fingerprint
   * 
   * @returns Promise resolving to canvas fingerprint
   */
  async collect(): Promise<CanvasFingerprint> {
    try {
      const canvas = this.createCanvas();
      const ctx = canvas.getContext('2d');

      if (!ctx) {
        return {
          hash: '',
          supported: false,
        };
      }

      // Render multi-layer fingerprinting pattern
      this.renderPattern(ctx);

      // Get image data and hash it
      const dataUrl = canvas.toDataURL();
      const hash = await sha256(dataUrl);

      return {
        hash,
        supported: true,
      };
    } catch (error) {
      return {
        hash: '',
        supported: false,
      };
    }
  }

  /**
   * Create canvas element
   * 
   * @private
   */
  private createCanvas(): HTMLCanvasElement {
    const canvas = document.createElement('canvas');
    canvas.width = 200;
    canvas.height = 50;
    return canvas;
  }

  /**
   * Render fingerprinting pattern on canvas
   * 
   * This uses multiple drawing operations to capture browser/GPU variations:
   * - Text rendering (font rendering differences)
   * - Geometric shapes (anti-aliasing differences)
   * - Color gradients (color space differences)
   * - Transparency (compositing differences)
   * 
   * @private
   */
  private renderPattern(ctx: CanvasRenderingContext2D): void {
    // Set baseline
    ctx.textBaseline = 'top';

    // Layer 1: Gradient rectangle
    const gradient = ctx.createLinearGradient(0, 0, 200, 50);
    gradient.addColorStop(0, '#f60');
    gradient.addColorStop(0.5, '#069');
    gradient.addColorStop(1, '#0f0');
    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, 200, 50);

    // Layer 2: Text with specific font
    ctx.font = '14px "Arial"';
    ctx.fillStyle = '#f60';
    ctx.fillRect(125, 1, 62, 20);

    // Layer 3: Overlapping text
    ctx.fillStyle = '#069';
    ctx.fillText('Scrybe 🦉', 2, 15);

    // Layer 4: Semi-transparent overlay
    ctx.fillStyle = 'rgba(102, 204, 0, 0.7)';
    ctx.fillText('Bot Detection', 4, 17);

    // Layer 5: Geometric shapes
    ctx.beginPath();
    ctx.arc(50, 25, 20, 0, Math.PI * 2, true);
    ctx.closePath();
    ctx.strokeStyle = '#ff0000';
    ctx.stroke();

    // Layer 6: Additional text variation
    ctx.font = '12px "Courier New"';
    ctx.fillStyle = '#000';
    ctx.fillText('FP Test', 120, 30);
  }
}
