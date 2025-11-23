import type { ScrybeConfig, TelemetryPayload, IngestResponse } from '../types';
export declare class HttpClient {
    private config;
    constructor(config: ScrybeConfig);
    send(payload: TelemetryPayload): Promise<IngestResponse>;
}
//# sourceMappingURL=http.d.ts.map