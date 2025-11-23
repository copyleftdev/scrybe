/**
 * Scrybe SDK - Lightweight Browser Fingerprinting
 * 
 * @packageDocumentation
 */

import type {
  ScrybeConfig,
  TelemetryPayload,
  IngestResponse,
} from './types';
import { StaticCollector } from './collectors/static';
import { CanvasCollector } from './collectors/canvas';
import { WebGLCollector } from './collectors/webgl';
import { AudioCollector } from './collectors/audio';
import { FontCollector } from './collectors/fonts';
import { BehavioralCollector } from './collectors/behavioral';
import { HttpClient } from './transport/http';
import { ConsentManager } from './privacy/consent';
import { generateSessionId } from './utils/session';

/**
 * Main Scrybe SDK class
 * 
 * @example
 * ```typescript
 * const scrybe = new Scrybe({
 *   apiUrl: 'https://api.scrybe.io',
 *   apiKey: 'your-api-key',
 *   consentGiven: true,
 * });
 * 
 * await scrybe.init();
 * ```
 */
export class Scrybe {
  private config: ScrybeConfig;
  private sessionId: string;
  private httpClient: HttpClient;
  private consentManager: ConsentManager;
  private isInitialized = false;

  constructor(config: ScrybeConfig) {
    this.config = {
      respectDoNotTrack: true,
      debug: false,
      timeout: 5000,
      ...config,
    };

    this.sessionId = generateSessionId();
    this.httpClient = new HttpClient(this.config);
    this.consentManager = new ConsentManager(this.config);
  }

  /**
   * Initialize the SDK and start collecting signals
   * 
   * @returns Promise that resolves when initialization is complete
   * @throws Error if consent is not given or Do Not Track is enabled
   */
  async init(): Promise<void> {
    // Check Do Not Track
    if (this.config.respectDoNotTrack && this.isDNTEnabled()) {
      this.log('Do Not Track enabled. Skipping initialization.');
      return;
    }

    // Check GDPR consent
    if (!this.consentManager.hasConsent()) {
      this.log('Consent not given. Skipping initialization.');
      return;
    }

    this.log('Initializing Scrybe SDK...');

    try {
      // Collect all signals asynchronously
      const payload = await this.collectSignals();

      // Send to ingestion endpoint
      await this.sendTelemetry(payload);

      this.isInitialized = true;
      this.log('Scrybe SDK initialized successfully');
    } catch (error) {
      this.log('Failed to initialize Scrybe SDK:', error);
      throw error;
    }
  }

  /**
   * Collect all signals from the browser
   * 
   * @private
   * @returns Complete telemetry payload
   */
  private async collectSignals(): Promise<TelemetryPayload> {
    const staticCollector = new StaticCollector();
    const canvasCollector = new CanvasCollector();
    const webglCollector = new WebGLCollector();
    const audioCollector = new AudioCollector();
    const fontCollector = new FontCollector();
    const behavioralCollector = new BehavioralCollector();

    // Collect signals in parallel
    const [staticSignals, canvasFingerprint, webglFingerprint, audioFingerprint, fontFingerprint] =
      await Promise.all([
        staticCollector.collect(),
        canvasCollector.collect(),
        webglCollector.collect(),
        audioCollector.collect(),
        fontCollector.collect(),
      ]);

    // Behavioral signals are collected asynchronously
    const behavioralSignals = behavioralCollector.getSignals();

    return {
      sessionId: this.sessionId,
      timestamp: Date.now(),
      network: staticSignals.network,
      browser: {
        ...staticSignals.browser,
        canvas: canvasFingerprint,
        webgl: webglFingerprint,
        audio: audioFingerprint,
        fonts: fontFingerprint,
      },
      behavioral: behavioralSignals,
    };
  }

  /**
   * Send telemetry to the ingestion endpoint
   * 
   * @private
   * @param payload - Telemetry data
   * @returns API response
   */
  private async sendTelemetry(payload: TelemetryPayload): Promise<IngestResponse> {
    return this.httpClient.send(payload);
  }

  /**
   * Check if Do Not Track is enabled
   * 
   * @private
   * @returns true if DNT is enabled
   */
  private isDNTEnabled(): boolean {
    return (
      navigator.doNotTrack === '1' ||
      (window as any).doNotTrack === '1' ||
      (navigator as any).msDoNotTrack === '1'
    );
  }

  /**
   * Log message if debug mode is enabled
   * 
   * @private
   * @param args - Arguments to log
   */
  private log(...args: any[]): void {
    if (this.config.debug) {
      console.log('[Scrybe]', ...args);
    }
  }

  /**
   * Update user consent status
   * 
   * @param granted - Whether consent is granted
   */
  setConsent(granted: boolean): void {
    this.consentManager.setConsent(granted);

    if (granted && !this.isInitialized) {
      this.init().catch((err) => this.log('Initialization failed:', err));
    }
  }

  /**
   * Get current session ID
   * 
   * @returns Current session identifier
   */
  getSessionId(): string {
    return this.sessionId;
  }
}

// Export types
export * from './types';

// Default export
export default Scrybe;
