import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-bugreport-thanks',
  standalone: true,
  imports: [CommonModule, RouterLink],
  templateUrl: './bugreport-thanks.component.html',
  styleUrl: './bugreport-thanks.component.css'
})
export class BugreportThanksComponent {}
