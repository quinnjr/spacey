import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';
import { SystemInfoService, SystemInfo } from '../../services/system-info.service';

@Component({
  selector: 'app-welcome',
  standalone: true,
  imports: [CommonModule, RouterLink],
  templateUrl: './welcome.component.html',
  styleUrl: './welcome.component.css'
})
export class WelcomeComponent implements OnInit {
  systemInfo: SystemInfo | null = null;

  features = [
    { icon: '✅', text: 'Custom JavaScript engine (Spacey)', status: 'complete' },
    { icon: '✅', text: 'Basic HTML rendering', status: 'complete' },
    { icon: '✅', text: 'GPU-accelerated graphics (wgpu)', status: 'complete' },
    { icon: '✅', text: 'AI-powered browsing assistant (Phi-3)', status: 'complete' },
    { icon: '✅', text: 'Firefox-compatible extensions', status: 'complete' },
    { icon: '✅', text: 'Full webRequest API (ad blockers work!)', status: 'complete' },
    { icon: '✅', text: 'Built-in privacy protection (Spacey Shield)', status: 'complete' },
    { icon: '🚧', text: 'CSS support', status: 'wip' },
    { icon: '🚧', text: 'Full DOM API', status: 'wip' },
  ];

  quickLinks = [
    { icon: '⚙️', title: 'Settings', description: 'Configure browser preferences', route: '/settings' },
    { icon: '🐛', title: 'Report Bug', description: 'Help us improve Spacey', route: '/bugreport' },
  ];

  aiCommands = [
    'Search for Rust tutorials',
    'Navigate to github.com',
    'Extract all headings from this page'
  ];

  constructor(private systemInfoService: SystemInfoService) {}

  ngOnInit() {
    this.systemInfo = this.systemInfoService.getSystemInfo();
  }
}
