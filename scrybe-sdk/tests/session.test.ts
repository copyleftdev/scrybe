import { generateSessionId, storeSessionId, getStoredSessionId } from '../src/utils/session';

describe('Session Utilities', () => {
  describe('generateSessionId', () => {
    it('should generate a valid UUID v4', () => {
      const id = generateSessionId();
      
      // UUID v4 format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
      const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
      expect(id).toMatch(uuidRegex);
    });

    it('should generate unique IDs', () => {
      const id1 = generateSessionId();
      const id2 = generateSessionId();
      
      expect(id1).not.toBe(id2);
    });
  });

  describe('session storage', () => {
    beforeEach(() => {
      sessionStorage.clear();
    });

    it('should store and retrieve session ID', () => {
      const testId = 'test-session-id';
      
      storeSessionId(testId);
      const retrieved = getStoredSessionId();
      
      expect(retrieved).toBe(testId);
    });

    it('should return null when no session ID stored', () => {
      const retrieved = getStoredSessionId();
      expect(retrieved).toBeNull();
    });
  });
});
