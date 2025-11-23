/**
 * Static signal collector - Collects non-changing browser properties
 */

import type {
  NetworkSignals,
  BrowserSignals,
  ScreenInfo,
  NavigatorInfo,
  AutomationQuirks,
  StorageInfo,
  PluginInfo,
} from '../types';

export class StaticCollector {
  /**
   * Collect all static signals
   * 
   * @returns Promise resolving to static signals
   */
  async collect(): Promise<{ network: NetworkSignals; browser: Partial<BrowserSignals> }> {
    return {
      network: this.collectNetworkSignals(),
      browser: {
        screen: this.collectScreenInfo(),
        navigator: this.collectNavigatorInfo(),
        quirks: this.detectAutomation(),
        storage: this.checkStorage(),
        plugins: this.collectPlugins(),
      },
    };
  }

  /**
   * Collect network-related signals
   * 
   * @private
   */
  private collectNetworkSignals(): NetworkSignals {
    const connection = (navigator as any).connection || (navigator as any).mozConnection || (navigator as any).webkitConnection;

    return {
      effectiveType: connection?.effectiveType,
      downlink: connection?.downlink,
      rtt: connection?.rtt,
      httpVersion: this.getHttpVersion(),
    };
  }

  /**
   * Get HTTP version from Performance API
   * 
   * @private
   */
  private getHttpVersion(): string | undefined {
    try {
      const entry = performance.getEntriesByType('navigation')[0] as any;
      return entry?.nextHopProtocol;
    } catch {
      return undefined;
    }
  }

  /**
   * Collect screen and viewport information
   * 
   * @private
   */
  private collectScreenInfo(): ScreenInfo {
    return {
      width: screen.width,
      height: screen.height,
      colorDepth: screen.colorDepth,
      pixelRatio: window.devicePixelRatio,
      orientation: screen.orientation?.type || 'unknown',
    };
  }

  /**
   * Collect navigator information
   * 
   * @private
   */
  private collectNavigatorInfo(): NavigatorInfo {
    return {
      userAgent: navigator.userAgent,
      language: navigator.language,
      languages: Array.from(navigator.languages || [navigator.language]),
      platform: navigator.platform,
      hardwareConcurrency: navigator.hardwareConcurrency,
      deviceMemory: (navigator as any).deviceMemory,
      maxTouchPoints: navigator.maxTouchPoints,
    };
  }

  /**
   * Detect browser automation and headless browsers
   * 
   * @private
   */
  private detectAutomation(): AutomationQuirks {
    return {
      webdriver: navigator.webdriver === true,
      automation: this.detectGenericAutomation(),
      phantom: this.detectPhantomJS(),
      selenium: this.detectSelenium(),
    };
  }

  /**
   * Detect generic automation indicators
   * 
   * @private
   */
  private detectGenericAutomation(): boolean {
    // Check for common automation properties
    const win = window as any;
    return !!(
      win._phantom ||
      win.__nightmare ||
      win._selenium ||
      win.callPhantom ||
      win.callSelenium ||
      win.__webdriver_evaluate ||
      win.__driver_evaluate ||
      win.__webdriver_script_function ||
      win.__webdriver_script_func ||
      win.__webdriver_script_fn ||
      win.__fxdriver_evaluate ||
      win.__driver_unwrapped ||
      win.__webdriver_unwrapped ||
      win.__selenium_unwrapped
    );
  }

  /**
   * Detect PhantomJS
   * 
   * @private
   */
  private detectPhantomJS(): boolean {
    const win = window as any;
    return !!(win._phantom || win.callPhantom);
  }

  /**
   * Detect Selenium
   * 
   * @private
   */
  private detectSelenium(): boolean {
    const win = window as any;
    const doc = document as any;
    return !!(
      win._selenium ||
      win.__selenium_unwrapped ||
      win.__webdriver_evaluate ||
      win.__driver_evaluate ||
      doc.__webdriver_evaluate ||
      doc.__selenium_unwrapped ||
      doc.__driver_unwrapped
    );
  }

  /**
   * Check storage availability
   * 
   * @private
   */
  private checkStorage(): StorageInfo {
    return {
      localStorage: this.isStorageAvailable('localStorage'),
      sessionStorage: this.isStorageAvailable('sessionStorage'),
      indexedDB: 'indexedDB' in window,
      cookies: navigator.cookieEnabled,
    };
  }

  /**
   * Test if a storage type is available
   * 
   * @private
   */
  private isStorageAvailable(type: 'localStorage' | 'sessionStorage'): boolean {
    try {
      const storage = window[type];
      const test = '__storage_test__';
      storage.setItem(test, test);
      storage.removeItem(test);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Collect plugin information
   * 
   * @private
   */
  private collectPlugins(): PluginInfo {
    const plugins = Array.from(navigator.plugins || []);
    return {
      count: plugins.length,
      list: plugins.map((p) => p.name).slice(0, 20), // Limit to prevent fingerprint bloat
    };
  }
}
