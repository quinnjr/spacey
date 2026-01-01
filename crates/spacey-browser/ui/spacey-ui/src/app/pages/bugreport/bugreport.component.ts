import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { SystemInfoService, SystemInfo } from '../../services/system-info.service';

interface BugReport {
  issueType: string;
  summary: string;
  url: string;
  steps: string;
  expected: string;
  actual: string;
  additional: string;
  email: string;
}

@Component({
  selector: 'app-bugreport',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './bugreport.component.html',
  styleUrl: './bugreport.component.css'
})
export class BugreportComponent implements OnInit {
  systemInfo: SystemInfo | null = null;
  
  issueTypes = [
    { value: 'crash', label: '💥 Crash / Freeze' },
    { value: 'rendering', label: '🎨 Rendering Issue' },
    { value: 'javascript', label: '⚡ JavaScript Error' },
    { value: 'extension', label: '🧩 Extension Problem' },
    { value: 'shield', label: '🛡️ Shield / Blocking Issue' },
    { value: 'ai', label: '🤖 AI Assistant Issue' },
    { value: 'performance', label: '🐢 Performance Problem' },
    { value: 'ui', label: '🖼️ UI / UX Issue' },
    { value: 'other', label: '📝 Other' }
  ];
  
  report: BugReport = {
    issueType: '',
    summary: '',
    url: '',
    steps: '',
    expected: '',
    actual: '',
    additional: '',
    email: ''
  };
  
  submitting = false;
  
  constructor(private systemInfoService: SystemInfoService) {}
  
  ngOnInit() {
    this.systemInfo = this.systemInfoService.getSystemInfo();
  }
  
  getFormAction(): string {
    return 'https://formsubmit.co/support@pegasusheavy.dev';
  }
  
  getSystemInfoString(): string {
    if (!this.systemInfo) return '';
    return `Spacey v${this.systemInfo.version} | ${this.systemInfo.os} (${this.systemInfo.arch}) | Shield: ${this.systemInfo.shieldLevel} | Extensions: ${this.systemInfo.extensionCount}`;
  }
  
  onSubmit() {
    this.submitting = true;
    // Form will submit via FormSubmit.co
  }
}
