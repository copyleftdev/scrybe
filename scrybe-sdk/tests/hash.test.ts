import { sha256, simpleHash } from '../src/utils/hash';

describe('Hash Utilities', () => {
  describe('sha256', () => {
    it('should generate consistent SHA-256 hash', async () => {
      const input = 'test-data';
      const hash1 = await sha256(input);
      const hash2 = await sha256(input);
      
      expect(hash1).toBe(hash2);
      expect(hash1).toHaveLength(64); // SHA-256 is 32 bytes = 64 hex chars
    });

    it('should generate different hashes for different inputs', async () => {
      const hash1 = await sha256('input1');
      const hash2 = await sha256('input2');
      
      expect(hash1).not.toBe(hash2);
    });
  });

  describe('simpleHash', () => {
    it('should generate consistent hash', () => {
      const input = 'test-data';
      const hash1 = simpleHash(input);
      const hash2 = simpleHash(input);
      
      expect(hash1).toBe(hash2);
    });

    it('should generate different hashes for different inputs', () => {
      const hash1 = simpleHash('input1');
      const hash2 = simpleHash('input2');
      
      expect(hash1).not.toBe(hash2);
    });

    it('should return a number', () => {
      const hash = simpleHash('test');
      expect(typeof hash).toBe('number');
    });
  });
});
