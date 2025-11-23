/**
 * HTTP client for sending telemetry to the ingestion endpoint
 */

import type { ScrybeConfig, TelemetryPayload, IngestResponse } from '../types';
import { generateNonce, signPayload } from '../security/signing';

export class HttpClient {
  private config: ScrybeConfig;

  constructor(config: ScrybeConfig) {
    this.config = config;
  }

  /**
   * Send telemetry payload to ingestion endpoint
   * 
   * @param payload - Telemetry data
   * @returns API response
   */
  async send(payload: TelemetryPayload): Promise<IngestResponse> {
    const timestamp = Date.now();
    const nonce = generateNonce();
    const body = JSON.stringify(payload);

    // Generate HMAC signature
    const signature = await signPayload(body, timestamp, nonce, this.config.apiKey);

    // Send request
    const response = await fetch(`${this.config.apiUrl}/api/v1/ingest`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Scrybe-Timestamp': timestamp.toString(),
        'X-Scrybe-Nonce': nonce,
        'X-Scrybe-Signature': signature,
      },
      body,
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json();
  }
}
