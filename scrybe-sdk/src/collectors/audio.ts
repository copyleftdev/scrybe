/**
 * Audio fingerprinting collector
 * 
 * Generates a unique fingerprint based on AudioContext characteristics
 * and audio processing capabilities.
 */

import type { AudioFingerprint } from '../types';
import { sha256 } from '../utils/hash';

export class AudioCollector {
  /**
   * Collect audio fingerprint
   * 
   * @returns Promise resolving to audio fingerprint
   */
  async collect(): Promise<AudioFingerprint | undefined> {
    try {
      // Check if AudioContext is available
      const AudioContext = (window as any).AudioContext || (window as any).webkitAudioContext;

      if (!AudioContext) {
        return undefined;
      }

      const context = new AudioContext();
      const oscillator = context.createOscillator();
      const analyser = context.createAnalyser();
      const gainNode = context.createGain();
      const scriptProcessor = context.createScriptProcessor(4096, 1, 1);

      // Set up audio processing chain
      gainNode.gain.value = 0; // Mute output
      oscillator.type = 'triangle';
      oscillator.frequency.value = 10000;

      oscillator.connect(analyser);
      analyser.connect(scriptProcessor);
      scriptProcessor.connect(gainNode);
      gainNode.connect(context.destination);

      // Collect audio fingerprint data
      const fingerprint = await new Promise<string>((resolve) => {
        scriptProcessor.onaudioprocess = (event) => {
          const output = event.outputBuffer.getChannelData(0);
          const fingerprint = this.computeFingerprint(output);
          resolve(fingerprint);
        };

        oscillator.start(0);
        setTimeout(() => {
          oscillator.stop();
        }, 100);
      });

      // Clean up
      context.close();

      const hash = await sha256(fingerprint);

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
   * Compute fingerprint from audio data
   * 
   * @private
   */
  private computeFingerprint(data: Float32Array): string {
    // Sample the audio data at regular intervals
    const samples: number[] = [];
    const sampleInterval = Math.floor(data.length / 30);

    for (let i = 0; i < data.length; i += sampleInterval) {
      samples.push(Math.abs(data[i] || 0));
    }

    // Convert to string with precision
    return samples.map((s) => s.toFixed(10)).join(',');
  }
}
