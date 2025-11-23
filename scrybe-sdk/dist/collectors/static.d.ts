import type { NetworkSignals, BrowserSignals } from '../types';
export declare class StaticCollector {
    collect(): Promise<{
        network: NetworkSignals;
        browser: Partial<BrowserSignals>;
    }>;
    private collectNetworkSignals;
    private getHttpVersion;
    private collectScreenInfo;
    private collectNavigatorInfo;
    private detectAutomation;
    private detectGenericAutomation;
    private detectPhantomJS;
    private detectSelenium;
    private checkStorage;
    private isStorageAvailable;
    private collectPlugins;
}
//# sourceMappingURL=static.d.ts.map