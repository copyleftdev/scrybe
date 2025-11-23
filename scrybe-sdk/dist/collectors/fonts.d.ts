import type { FontFingerprint } from '../types';
export declare class FontCollector {
    private readonly testFonts;
    private readonly baseFonts;
    private readonly testString;
    private readonly testSize;
    collect(): Promise<FontFingerprint | undefined>;
    private detectFonts;
}
//# sourceMappingURL=fonts.d.ts.map