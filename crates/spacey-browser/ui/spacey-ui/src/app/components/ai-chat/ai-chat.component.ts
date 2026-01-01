import { Component, Input, Output, EventEmitter, ViewChild, ElementRef, AfterViewChecked } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { AiProviderService, AiProviderConfig } from '../../services/ai-provider.service';

/**
 * Represents a single message in the AI chat
 */
export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  thinking?: string;
  thinkingComplete?: boolean;
  timestamp: Date;
  isStreaming?: boolean;
  toolUsed?: string;
  toolResult?: string;
}

/**
 * AI Chat component with thinking mode support
 */
@Component({
  selector: 'app-ai-chat',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './ai-chat.component.html',
  styleUrl: './ai-chat.component.css'
})
export class AiChatComponent implements AfterViewChecked {
  @Input() messages: ChatMessage[] = [];
  @Input() isProcessing = false;
  @Output() sendMessage = new EventEmitter<string>();
  @Output() cancelRequest = new EventEmitter<void>();

  @ViewChild('messagesContainer') messagesContainer!: ElementRef;
  @ViewChild('messageInput') messageInput!: ElementRef;

  userMessage = '';
  expandedThinking: Set<string> = new Set();
  config: AiProviderConfig;

  constructor(private aiProviderService: AiProviderService) {
    this.config = this.aiProviderService.getConfig();
  }

  ngAfterViewChecked() {
    this.scrollToBottom();
  }

  private scrollToBottom(): void {
    if (this.messagesContainer) {
      const el = this.messagesContainer.nativeElement;
      el.scrollTop = el.scrollHeight;
    }
  }

  onSend(): void {
    const message = this.userMessage.trim();
    if (message && !this.isProcessing) {
      this.sendMessage.emit(message);
      this.userMessage = '';
    }
  }

  onKeyDown(event: KeyboardEvent): void {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      this.onSend();
    }
  }

  onCancel(): void {
    this.cancelRequest.emit();
  }

  toggleThinking(messageId: string): void {
    if (this.expandedThinking.has(messageId)) {
      this.expandedThinking.delete(messageId);
    } else {
      this.expandedThinking.add(messageId);
    }
  }

  isThinkingExpanded(messageId: string): boolean {
    // Streaming style always shows expanded
    if (this.config.thinkingStyle === 'streaming') return true;
    // Expanded style always shows
    if (this.config.thinkingStyle === 'expanded') return true;
    // Collapsed style depends on user action
    return this.expandedThinking.has(messageId);
  }

  shouldShowThinking(message: ChatMessage): boolean {
    return this.config.showThinking && !!message.thinking;
  }

  getProviderIcon(): string {
    switch (this.config.provider) {
      case 'claude': return '🟣';
      case 'openai': return '🟢';
      default: return '🖥️';
    }
  }

  getProviderName(): string {
    switch (this.config.provider) {
      case 'claude': return 'Claude';
      case 'openai': return 'ChatGPT';
      default: return 'Local AI';
    }
  }

  formatThinkingTime(thinking: string): string {
    // Count approximate tokens (rough estimate)
    const wordCount = thinking.split(/\s+/).length;
    const estimatedTime = Math.ceil(wordCount / 100); // ~100 words per second reading
    return estimatedTime < 60 ? `${estimatedTime}s` : `${Math.ceil(estimatedTime / 60)}m`;
  }

  /**
   * Generate a unique ID for messages
   */
  static generateId(): string {
    return `msg-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
  }
}
