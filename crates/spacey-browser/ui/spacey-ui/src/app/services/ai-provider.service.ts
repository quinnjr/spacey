import { Injectable } from '@angular/core';

/**
 * Supported AI providers
 */
export type AiProviderType = 'local' | 'claude' | 'openai';

/**
 * AI Provider configuration
 */
export interface AiProviderConfig {
  /** Whether AI features are enabled at all */
  enabled: boolean;
  /** Whether the local model is enabled (if false, saves resources) */
  localEnabled: boolean;
  /** Active provider */
  provider: AiProviderType;
  /** Model to use */
  model?: string;
  /** API key for cloud providers */
  apiKey?: string;
}

/**
 * Claude-specific models
 */
export const CLAUDE_MODELS = [
  { id: 'claude-sonnet-4-20250514', name: 'Claude Sonnet 4', description: 'Best balance of speed and capability' },
  { id: 'claude-opus-4-20250514', name: 'Claude Opus 4', description: 'Most capable, best for complex tasks' },
  { id: 'claude-3-5-sonnet-20241022', name: 'Claude 3.5 Sonnet', description: 'Previous generation, still excellent' },
  { id: 'claude-3-5-haiku-20241022', name: 'Claude 3.5 Haiku', description: 'Fastest, most economical' },
];

/**
 * OpenAI-specific models
 */
export const OPENAI_MODELS = [
  { id: 'gpt-4o', name: 'GPT-4o', description: 'Most capable, multimodal' },
  { id: 'gpt-4o-mini', name: 'GPT-4o Mini', description: 'Fast and affordable' },
  { id: 'gpt-4-turbo', name: 'GPT-4 Turbo', description: 'Previous flagship model' },
  { id: 'gpt-3.5-turbo', name: 'GPT-3.5 Turbo', description: 'Fast, economical' },
];

/**
 * Local model options
 */
export const LOCAL_MODELS = [
  { id: 'phi-3-mini-4k', name: 'Phi-3 Mini 4K', description: 'Default, ~4GB RAM' },
  { id: 'phi-3-mini-128k', name: 'Phi-3 Mini 128K', description: 'Extended context, ~8GB RAM' },
];

/**
 * Provider display info
 */
export const PROVIDER_INFO: Record<AiProviderType, {
  name: string;
  icon: string;
  description: string;
  requiresKey: boolean;
  keyPrefix?: string;
}> = {
  local: {
    name: 'Local (Phi-3)',
    icon: '🖥️',
    description: 'Runs entirely on your device. Free, private, no API key needed.',
    requiresKey: false,
  },
  claude: {
    name: 'Claude (Anthropic)',
    icon: '🟣',
    description: 'Anthropic\'s Claude models. Requires API key.',
    requiresKey: true,
    keyPrefix: 'sk-ant-',
  },
  openai: {
    name: 'ChatGPT (OpenAI)',
    icon: '🟢',
    description: 'OpenAI\'s GPT models. Requires API key.',
    requiresKey: true,
    keyPrefix: 'sk-',
  },
};

// Window.spaceyBridge interface is extended in system-info.service.ts

@Injectable({
  providedIn: 'root'
})
export class AiProviderService {
  private readonly STORAGE_KEY = 'spacey_ai_config';
  
  private readonly DEFAULT_CONFIG: AiProviderConfig = {
    enabled: true,
    localEnabled: true,
    provider: 'local',
    model: 'phi-3-mini-4k'
  };

  /**
   * Get the current AI configuration
   */
  getConfig(): AiProviderConfig {
    // Try browser bridge first
    if (typeof window !== 'undefined' && window.spaceyBridge?.getAiConfig) {
      return window.spaceyBridge.getAiConfig();
    }

    // Fallback to localStorage for development
    const stored = this.getStoredConfig();
    return stored || { ...this.DEFAULT_CONFIG };
  }

  /**
   * Save AI configuration
   */
  setConfig(config: AiProviderConfig): void {
    // Use browser bridge if available
    if (typeof window !== 'undefined' && window.spaceyBridge?.setAiConfig) {
      window.spaceyBridge.setAiConfig(config);
      return;
    }

    // Fallback to localStorage with basic obfuscation
    const toStore = { ...config };
    if (toStore.apiKey) {
      toStore.apiKey = this.obfuscateKey(toStore.apiKey);
    }
    localStorage.setItem(this.STORAGE_KEY, JSON.stringify(toStore));
  }
  
  /**
   * Check if AI is enabled
   */
  isEnabled(): boolean {
    return this.getConfig().enabled;
  }
  
  /**
   * Enable or disable AI features entirely
   */
  setEnabled(enabled: boolean): void {
    const config = this.getConfig();
    config.enabled = enabled;
    this.setConfig(config);
  }
  
  /**
   * Check if local model is enabled
   */
  isLocalEnabled(): boolean {
    return this.getConfig().localEnabled;
  }
  
  /**
   * Enable or disable the local model
   * When disabled, the model is unloaded to free GPU/CPU memory
   */
  setLocalEnabled(enabled: boolean): void {
    const config = this.getConfig();
    config.localEnabled = enabled;
    
    // If disabling local and it's the current provider, switch to another if available
    if (!enabled && config.provider === 'local') {
      // Check if we have other providers configured
      if (this.hasApiKey('claude')) {
        config.provider = 'claude';
      } else if (this.hasApiKey('openai')) {
        config.provider = 'openai';
      }
      // Otherwise stay on local (will show as disabled in UI)
    }
    
    this.setConfig(config);
    
    // Notify browser to unload the model if available
    if (typeof window !== 'undefined' && window.spaceyBridge?.setLocalModelEnabled) {
      window.spaceyBridge.setLocalModelEnabled(enabled);
    }
  }
  
  /**
   * Get available providers (those that can be used)
   */
  getAvailableProviders(): AiProviderType[] {
    const config = this.getConfig();
    const providers: AiProviderType[] = [];
    
    if (config.localEnabled) {
      providers.push('local');
    }
    
    if (this.hasApiKey('claude')) {
      providers.push('claude');
    }
    
    if (this.hasApiKey('openai')) {
      providers.push('openai');
    }
    
    return providers;
  }

  /**
   * Test if an API key is valid
   */
  async testApiKey(provider: AiProviderType, apiKey: string): Promise<{ valid: boolean; error?: string }> {
    // Use browser bridge if available
    if (typeof window !== 'undefined' && window.spaceyBridge?.testApiKey) {
      return window.spaceyBridge.testApiKey(provider, apiKey);
    }

    // Basic format validation
    const info = PROVIDER_INFO[provider];
    if (info.keyPrefix && !apiKey.startsWith(info.keyPrefix)) {
      return { valid: false, error: `API key should start with "${info.keyPrefix}"` };
    }

    if (apiKey.length < 30) {
      return { valid: false, error: 'API key appears to be too short' };
    }

    // Simulate API test in dev mode
    console.log(`[Dev Mode] Testing ${provider} API key...`);
    await new Promise(resolve => setTimeout(resolve, 500));
    return { valid: true };
  }

  /**
   * Clear stored API key
   */
  clearApiKey(provider: AiProviderType): void {
    const config = this.getConfig();
    if (config.provider === provider) {
      delete config.apiKey;
      this.setConfig(config);
    }
  }

  /**
   * Check if a provider has an API key configured
   */
  hasApiKey(provider: AiProviderType): boolean {
    if (provider === 'local') return true;
    const config = this.getStoredConfig();
    return config?.provider === provider && !!config?.apiKey;
  }

  /**
   * Get models for a provider
   */
  getModels(provider: AiProviderType): { id: string; name: string; description: string }[] {
    switch (provider) {
      case 'claude':
        return CLAUDE_MODELS;
      case 'openai':
        return OPENAI_MODELS;
      case 'local':
      default:
        return LOCAL_MODELS;
    }
  }

  /**
   * Get provider info
   */
  getProviderInfo(provider: AiProviderType) {
    return PROVIDER_INFO[provider];
  }

  /**
   * Get all providers
   */
  getAllProviders(): AiProviderType[] {
    return ['local', 'claude', 'openai'];
  }

  // Private helpers

  private getStoredConfig(): AiProviderConfig | null {
    if (typeof window === 'undefined') return null;

    try {
      const stored = localStorage.getItem(this.STORAGE_KEY);
      if (!stored) return null;

      const config = JSON.parse(stored) as Partial<AiProviderConfig>;
      
      // Merge with defaults to handle missing fields from older configs
      const merged: AiProviderConfig = {
        enabled: config.enabled ?? true,
        localEnabled: config.localEnabled ?? true,
        provider: config.provider ?? 'local',
        model: config.model,
        apiKey: config.apiKey,
      };
      
      // Deobfuscate key if present
      if (merged.apiKey) {
        merged.apiKey = this.deobfuscateKey(merged.apiKey);
      }
      return merged;
    } catch {
      return null;
    }
  }

  // Basic obfuscation for development - real encryption handled by Rust backend
  private obfuscateKey(key: string): string {
    return btoa(key.split('').reverse().join(''));
  }

  private deobfuscateKey(obfuscated: string): string {
    try {
      return atob(obfuscated).split('').reverse().join('');
    } catch {
      return '';
    }
  }
}
