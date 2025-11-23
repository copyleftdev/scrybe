/**
 * Consent management for GDPR compliance
 */

import type { ScrybeConfig } from '../types';

export class ConsentManager {
  private config: ScrybeConfig;
  private consentGiven: boolean;

  constructor(config: ScrybeConfig) {
    this.config = config;
    this.consentGiven = config.consentGiven || false;
  }

  /**
   * Check if user has given consent
   * 
   * @returns true if consent is given
   */
  hasConsent(): boolean {
    return this.consentGiven;
  }

  /**
   * Update consent status
   * 
   * @param granted - Whether consent is granted
   */
  setConsent(granted: boolean): void {
    this.consentGiven = granted;

    // Store consent in localStorage
    try {
      localStorage.setItem('scrybe_consent', granted ? '1' : '0');
    } catch {
      // Ignore storage errors
    }
  }

  /**
   * Check if visitor is from EU (heuristic)
   * 
   * Uses timezone as a simple heuristic. Not 100% accurate but useful
   * for determining if GDPR consent is likely required.
   * 
   * @returns true if likely EU visitor
   */
  isEUVisitor(): boolean {
    try {
      const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
      const euTimezones = ['Europe/', 'GMT', 'UTC', 'WET', 'CET', 'EET'];

      return euTimezones.some((tz) => timezone.startsWith(tz));
    } catch {
      return false;
    }
  }

  /**
   * Get stored consent from localStorage
   * 
   * @returns Stored consent status or null
   */
  getStoredConsent(): boolean | null {
    try {
      const stored = localStorage.getItem('scrybe_consent');
      if (stored === '1') return true;
      if (stored === '0') return false;
      return null;
    } catch {
      return null;
    }
  }
}
