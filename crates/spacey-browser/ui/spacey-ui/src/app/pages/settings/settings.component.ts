import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { SystemInfoService, SystemInfo } from '../../services/system-info.service';
import {
  AiProviderService,
  AiProviderType,
  AiProviderConfig,
  PROVIDER_INFO,
  CLAUDE_MODELS,
  OPENAI_MODELS,
  LOCAL_MODELS
} from '../../services/ai-provider.service';

interface SettingsSection {
  id: string;
  icon: string;
  title: string;
}

@Component({
  selector: 'app-settings',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './settings.component.html',
  styleUrl: './settings.component.css'
})
export class SettingsComponent implements OnInit {
  systemInfo: SystemInfo | null = null;
  activeSection = 'shield';

  sections: SettingsSection[] = [
    { id: 'shield', icon: '🛡️', title: 'Spacey Shield' },
    { id: 'ai', icon: '🤖', title: 'AI Assistant' },
    { id: 'extensions', icon: '🧩', title: 'Extensions' },
    { id: 'privacy', icon: '🔒', title: 'Privacy' },
    { id: 'appearance', icon: '🎨', title: 'Appearance' },
    { id: 'advanced', icon: '⚙️', title: 'Advanced' },
  ];

  // Shield settings
  shieldLevel: 'off' | 'standard' | 'strict' = 'standard';
  shieldOptions = {
    blockAds: true,
    blockTrackers: true,
    blockCryptominers: true,
    fingerprintProtection: true,
    httpsUpgrade: true,
    stripTracking: true,
  };

  // AI settings
  aiSettings = {
    enabled: true,
    localEnabled: true,
    autoLoad: false,
    maxIterations: 10,
    showThoughts: true,
  };

  // AI Provider settings (BYOK)
  aiProvider: AiProviderType = 'local';
  aiModel: string = 'phi-3-mini-4k';
  aiApiKey: string = '';
  aiApiKeyVisible: boolean = false;
  aiKeyTestStatus: 'idle' | 'testing' | 'valid' | 'invalid' = 'idle';
  aiKeyTestError: string = '';

  // Provider info for templates
  providers = Object.entries(PROVIDER_INFO).map(([id, info]) => ({ id: id as AiProviderType, ...info }));
  claudeModels = CLAUDE_MODELS;
  openaiModels = OPENAI_MODELS;
  localModels = LOCAL_MODELS;

  // Privacy settings
  privacySettings = {
    doNotTrack: true,
    globalPrivacyControl: true,
    clearOnExit: false,
    blockThirdPartyCookies: true,
  };

  // Appearance settings
  appearanceSettings = {
    theme: 'dark',
    fontSize: 'medium',
    showBookmarksBar: true,
    compactMode: false,
  };

  // Advanced settings
  advancedSettings = {
    hardwareAcceleration: true,
    experimentalFeatures: false,
    developerMode: false,
  };

  constructor(
    private systemInfoService: SystemInfoService,
    private aiProviderService: AiProviderService
  ) {}

  ngOnInit() {
    this.systemInfo = this.systemInfoService.getSystemInfo();
    if (this.systemInfo) {
      this.shieldLevel = this.systemInfo.shieldLevel.toLowerCase() as 'off' | 'standard' | 'strict';
    }

    // Load AI provider config
    const aiConfig = this.aiProviderService.getConfig();
    this.aiProvider = aiConfig.provider;
    this.aiModel = aiConfig.model || this.getDefaultModel(aiConfig.provider);
    this.aiSettings.enabled = aiConfig.enabled;
    this.aiSettings.localEnabled = aiConfig.localEnabled;
    
    if (aiConfig.apiKey) {
      this.aiApiKey = aiConfig.apiKey;
      this.aiKeyTestStatus = 'valid'; // Assume stored key is valid
    }
  }

  setActiveSection(id: string) {
    this.activeSection = id;
  }

  setShieldLevel(level: 'off' | 'standard' | 'strict') {
    this.shieldLevel = level;
    this.saveSettings();
  }

  onToggle(setting: string, section: string) {
    this.saveSettings();
  }

  // AI Provider methods

  selectProvider(provider: AiProviderType) {
    this.aiProvider = provider;
    this.aiModel = this.getDefaultModel(provider);
    this.aiApiKey = '';
    this.aiKeyTestStatus = 'idle';
    this.aiKeyTestError = '';

    // If local, save immediately
    if (provider === 'local') {
      this.saveAiConfig();
    }
  }

  getDefaultModel(provider: AiProviderType): string {
    switch (provider) {
      case 'claude': return 'claude-sonnet-4-20250514';
      case 'openai': return 'gpt-4o';
      default: return 'phi-3-mini-4k';
    }
  }

  getModelsForProvider(): { id: string; name: string; description: string }[] {
    return this.aiProviderService.getModels(this.aiProvider);
  }

  toggleApiKeyVisibility() {
    this.aiApiKeyVisible = !this.aiApiKeyVisible;
  }

  async testApiKey() {
    if (!this.aiApiKey) {
      this.aiKeyTestStatus = 'invalid';
      this.aiKeyTestError = 'Please enter an API key';
      return;
    }

    this.aiKeyTestStatus = 'testing';
    this.aiKeyTestError = '';

    try {
      const result = await this.aiProviderService.testApiKey(this.aiProvider, this.aiApiKey);

      if (result.valid) {
        this.aiKeyTestStatus = 'valid';
        this.saveAiConfig();
      } else {
        this.aiKeyTestStatus = 'invalid';
        this.aiKeyTestError = result.error || 'Invalid API key';
      }
    } catch (error) {
      this.aiKeyTestStatus = 'invalid';
      this.aiKeyTestError = 'Failed to test API key';
    }
  }

  saveAiConfig() {
    const config: AiProviderConfig = {
      enabled: this.aiSettings.enabled,
      localEnabled: this.aiSettings.localEnabled,
      provider: this.aiProvider,
      model: this.aiModel,
    };

    if (this.aiProvider !== 'local' && this.aiApiKey) {
      config.apiKey = this.aiApiKey;
    }

    this.aiProviderService.setConfig(config);
  }
  
  /**
   * Handle toggling the local model on/off
   */
  onLocalModelToggle() {
    // If disabling local model while it's the active provider
    if (!this.aiSettings.localEnabled && this.aiProvider === 'local') {
      // Check if we have cloud providers configured
      if (this.aiProviderService.hasApiKey('claude')) {
        this.aiProvider = 'claude';
        this.aiModel = this.getDefaultModel('claude');
      } else if (this.aiProviderService.hasApiKey('openai')) {
        this.aiProvider = 'openai';
        this.aiModel = this.getDefaultModel('openai');
      }
      // Otherwise stay on local (but it won't work until re-enabled)
    }
    
    // Save the configuration
    this.aiProviderService.setLocalEnabled(this.aiSettings.localEnabled);
    this.saveAiConfig();
  }

  clearApiKey() {
    this.aiApiKey = '';
    this.aiKeyTestStatus = 'idle';
    this.aiKeyTestError = '';
    this.aiProviderService.clearApiKey(this.aiProvider);
  }

  getProviderStatusClass(): string {
    if (this.aiProvider === 'local') return 'text-green-400';
    if (this.aiKeyTestStatus === 'valid') return 'text-green-400';
    if (this.aiKeyTestStatus === 'invalid') return 'text-red-400';
    return 'text-yellow-400';
  }

  getProviderStatusText(): string {
    if (this.aiProvider === 'local') return 'READY';
    if (this.aiKeyTestStatus === 'valid') return 'CONNECTED';
    if (this.aiKeyTestStatus === 'invalid') return 'ERROR';
    if (this.aiKeyTestStatus === 'testing') return 'TESTING...';
    return 'KEY REQUIRED';
  }

  saveSettings() {
    console.log('Settings saved:', {
      shield: { level: this.shieldLevel, ...this.shieldOptions },
      ai: this.aiSettings,
      privacy: this.privacySettings,
      appearance: this.appearanceSettings,
      advanced: this.advancedSettings,
    });
  }

  resetToDefaults() {
    this.shieldLevel = 'standard';
    this.shieldOptions = {
      blockAds: true,
      blockTrackers: true,
      blockCryptominers: true,
      fingerprintProtection: true,
      httpsUpgrade: true,
      stripTracking: true,
    };
    this.aiSettings = {
      enabled: true,
      localEnabled: true,
      autoLoad: false,
      maxIterations: 10,
      showThoughts: true,
    };
    this.aiProvider = 'local';
    this.aiModel = 'phi-3-mini-4k';
    this.aiApiKey = '';
    this.aiKeyTestStatus = 'idle';
    this.saveAiConfig();
    this.saveSettings();
  }
}
