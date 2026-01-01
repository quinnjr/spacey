import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { SystemInfoService, SystemInfo } from '../../services/system-info.service';

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
    autoLoad: false,
    maxIterations: 10,
    showThoughts: true,
  };

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

  constructor(private systemInfoService: SystemInfoService) {}

  ngOnInit() {
    this.systemInfo = this.systemInfoService.getSystemInfo();
    if (this.systemInfo) {
      this.shieldLevel = this.systemInfo.shieldLevel.toLowerCase() as 'off' | 'standard' | 'strict';
    }
  }

  setActiveSection(id: string) {
    this.activeSection = id;
  }

  setShieldLevel(level: 'off' | 'standard' | 'strict') {
    this.shieldLevel = level;
    // In a real app, this would communicate with the browser backend
    this.saveSettings();
  }

  onToggle(setting: string, section: string) {
    // Save settings when toggled
    this.saveSettings();
  }

  saveSettings() {
    // In a real implementation, this would send settings to the browser backend
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
      autoLoad: false,
      maxIterations: 10,
      showThoughts: true,
    };
    this.saveSettings();
  }
}
