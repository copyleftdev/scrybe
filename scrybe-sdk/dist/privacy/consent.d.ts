import type { ScrybeConfig } from '../types';
export declare class ConsentManager {
    private config;
    private consentGiven;
    constructor(config: ScrybeConfig);
    hasConsent(): boolean;
    setConsent(granted: boolean): void;
    isEUVisitor(): boolean;
    getStoredConsent(): boolean | null;
}
//# sourceMappingURL=consent.d.ts.map