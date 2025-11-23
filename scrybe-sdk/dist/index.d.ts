import type { ScrybeConfig } from './types';
export declare class Scrybe {
    private config;
    private sessionId;
    private httpClient;
    private consentManager;
    private isInitialized;
    constructor(config: ScrybeConfig);
    init(): Promise<void>;
    private collectSignals;
    private sendTelemetry;
    private isDNTEnabled;
    private log;
    setConsent(granted: boolean): void;
    getSessionId(): string;
}
export * from './types';
export default Scrybe;
//# sourceMappingURL=index.d.ts.map