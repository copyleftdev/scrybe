import { ConsentManager } from '../src/privacy/consent';
import type { ScrybeConfig } from '../src/types';

describe('ConsentManager', () => {
  let config: ScrybeConfig;

  beforeEach(() => {
    config = {
      apiUrl: 'https://api.scrybe.test',
      apiKey: 'test-key',
      consentGiven: false,
    };
    localStorage.clear();
  });

  describe('hasConsent', () => {
    it('should return false when consent not given', () => {
      const manager = new ConsentManager(config);
      expect(manager.hasConsent()).toBe(false);
    });

    it('should return true when consent given in config', () => {
      config.consentGiven = true;
      const manager = new ConsentManager(config);
      expect(manager.hasConsent()).toBe(true);
    });
  });

  describe('setConsent', () => {
    it('should update consent status', () => {
      const manager = new ConsentManager(config);
      
      manager.setConsent(true);
      expect(manager.hasConsent()).toBe(true);
      
      manager.setConsent(false);
      expect(manager.hasConsent()).toBe(false);
    });

    it('should store consent in localStorage', () => {
      const manager = new ConsentManager(config);
      
      manager.setConsent(true);
      expect(localStorage.getItem('scrybe_consent')).toBe('1');
      
      manager.setConsent(false);
      expect(localStorage.getItem('scrybe_consent')).toBe('0');
    });
  });

  describe('getStoredConsent', () => {
    it('should return null when no consent stored', () => {
      const manager = new ConsentManager(config);
      expect(manager.getStoredConsent()).toBeNull();
    });

    it('should return stored consent value', () => {
      const manager = new ConsentManager(config);
      
      localStorage.setItem('scrybe_consent', '1');
      expect(manager.getStoredConsent()).toBe(true);
      
      localStorage.setItem('scrybe_consent', '0');
      expect(manager.getStoredConsent()).toBe(false);
    });
  });
});
