import { Injectable } from '@angular/core';

export interface SystemInfo {
  version: string;
  os: string;
  arch: string;
  shieldLevel: string;
  extensionCount: number;
  blockedDomains: number;
}

export interface AiProviderConfig {
  provider: 'local' | 'claude' | 'openai';
  model?: string;
  apiKey?: string;
}

declare global {
  interface Window {
    spaceyBridge?: {
      // System info
      getSystemInfo: () => SystemInfo;
      setShieldLevel: (level: string) => void;
      saveSettings: (settings: any) => void;
      // AI provider (BYOK)
      getAiConfig?: () => AiProviderConfig;
      setAiConfig?: (config: AiProviderConfig) => void;
      testApiKey?: (provider: string, apiKey: string) => Promise<{ valid: boolean; error?: string }>;
    };
  }
}

@Injectable({
  providedIn: 'root'
})
export class SystemInfoService {

  /**
   * Get system information from the browser bridge or return defaults.
   * In a real implementation, this would communicate with the Rust backend.
   */
  getSystemInfo(): SystemInfo {
    // Check if the browser bridge is available
    if (typeof window !== 'undefined' && window.spaceyBridge) {
      return window.spaceyBridge.getSystemInfo();
    }

    // Return default values for development/preview
    return {
      version: '0.1.0',
      os: this.detectOS(),
      arch: this.detectArch(),
      shieldLevel: 'Standard',
      extensionCount: 0,
      blockedDomains: 127,
    };
  }

  /**
   * Set the Shield protection level
   */
  setShieldLevel(level: string): void {
    if (typeof window !== 'undefined' && window.spaceyBridge) {
      window.spaceyBridge.setShieldLevel(level);
    } else {
      console.log('[Dev Mode] Setting shield level:', level);
    }
  }

  /**
   * Save settings to the browser backend
   */
  saveSettings(settings: any): void {
    if (typeof window !== 'undefined' && window.spaceyBridge) {
      window.spaceyBridge.saveSettings(settings);
    } else {
      console.log('[Dev Mode] Saving settings:', settings);
      // In development, save to localStorage
      localStorage.setItem('spacey_settings', JSON.stringify(settings));
    }
  }

  /**
   * Load settings from storage
   */
  loadSettings(): any {
    if (typeof window !== 'undefined') {
      const stored = localStorage.getItem('spacey_settings');
      if (stored) {
        try {
          return JSON.parse(stored);
        } catch {
          return null;
        }
      }
    }
    return null;
  }

  private detectOS(): string {
    if (typeof navigator === 'undefined') return 'Unknown';

    const userAgent = navigator.userAgent.toLowerCase();
    if (userAgent.includes('win')) return 'Windows';
    if (userAgent.includes('mac')) return 'macOS';
    if (userAgent.includes('linux')) return 'Linux';
    if (userAgent.includes('android')) return 'Android';
    if (userAgent.includes('ios')) return 'iOS';
    return 'Unknown';
  }

  private detectArch(): string {
    if (typeof navigator === 'undefined') return 'Unknown';

    // Modern browsers expose this via userAgentData
    const nav = navigator as any;
    if (nav.userAgentData?.platform) {
      return nav.userAgentData.platform.includes('64') ? 'x86_64' : 'x86';
    }

    // Fallback detection
    const userAgent = navigator.userAgent;
    if (userAgent.includes('x86_64') || userAgent.includes('x64') || userAgent.includes('Win64') || userAgent.includes('WOW64')) {
      return 'x86_64';
    }
    if (userAgent.includes('arm64') || userAgent.includes('aarch64')) {
      return 'aarch64';
    }
    return 'x86_64'; // Default assumption
  }
}
