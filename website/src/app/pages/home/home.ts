import { Component, signal } from '@angular/core';
import { RouterLink } from '@angular/router';
import { CommonModule } from '@angular/common';
import { FontAwesomeModule, FaIconLibrary } from '@fortawesome/angular-fontawesome';
import {
  faShieldHalved,
  faBolt,
  faCubes,
  faWandMagicSparkles,
  faFlask,
  faGlobe,
  faCheck,
  faSpinner,
  faClock,
  faBars,
  faXmark,
  faArrowRight,
  faEnvelope,
  faCopy,
  faChartLine
} from '@fortawesome/free-solid-svg-icons';
import { faGithub, faRust, faNode } from '@fortawesome/free-brands-svg-icons';

interface Star {
  id: number;
  x: number;
  y: number;
  delay: number;
  opacity: number;
}

interface StatusItem {
  name: string;
  description: string;
  status: 'complete' | 'in-progress' | 'planned';
}

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [RouterLink, CommonModule, FontAwesomeModule],
  templateUrl: './home.html',
  styleUrl: './home.css'
})
export class HomeComponent {
  protected readonly mobileMenuOpen = signal(false);

  // Font Awesome icons
  faGithub = faGithub;
  faShieldHalved = faShieldHalved;
  faBolt = faBolt;
  faCubes = faCubes;
  faWandMagicSparkles = faWandMagicSparkles;
  faFlask = faFlask;
  faGlobe = faGlobe;
  faCheck = faCheck;
  faSpinner = faSpinner;
  faClock = faClock;
  faBars = faBars;
  faXmark = faXmark;
  faArrowRight = faArrowRight;
  faEnvelope = faEnvelope;
  faCopy = faCopy;
  faRust = faRust;
  faNode = faNode;
  faChartLine = faChartLine;

  // Generate random stars for the background
  protected readonly stars: Star[] = Array.from({ length: 100 }, (_, i) => ({
    id: i,
    x: Math.random() * 100,
    y: Math.random() * 100,
    delay: Math.random() * 5,
    opacity: 0.3 + Math.random() * 0.7
  }));

  protected readonly statusItems: StatusItem[] = [
    { name: 'Lexer', description: 'Tokenization of JavaScript/TypeScript source code', status: 'complete' },
    { name: 'Parser', description: 'AST generation with full ES3 support', status: 'complete' },
    { name: 'Bytecode Compiler', description: 'Two-pass compilation with hoisting', status: 'complete' },
    { name: 'VM Interpreter', description: 'Stack-based bytecode execution', status: 'complete' },
    { name: 'Built-in Objects', description: 'Object, Array, String, Number, Date, RegExp, Math', status: 'complete' },
    { name: 'Garbage Collector', description: 'Generational GC with incremental marking', status: 'complete' },
    { name: 'TypeScript Support', description: 'Native TS parsing without transpilation', status: 'complete' },
    { name: 'Node.js APIs', description: 'CommonJS, ESM, fs, path, http, and more', status: 'complete' },
    { name: 'ES5 Compliance', description: 'Strict mode, getters/setters, JSON', status: 'in-progress' },
    { name: 'ES6+ Features', description: 'Classes, generators, async/await', status: 'planned' },
    { name: 'JIT Compiler', description: 'Just-in-time compilation for hot paths', status: 'planned' },
    { name: 'Servo Integration', description: 'Direct integration with Servo browser', status: 'planned' },
  ];

  constructor(library: FaIconLibrary) {
    // Add icons to the library
    library.addIcons(
      faGithub, faShieldHalved, faBolt, faCubes, faWandMagicSparkles,
      faFlask, faGlobe, faCheck, faSpinner, faClock, faBars, faXmark,
      faArrowRight, faEnvelope, faCopy, faRust, faNode, faChartLine
    );
  }

  toggleMobileMenu() {
    this.mobileMenuOpen.update(v => !v);
  }

  copyCode(section: string) {
    const codeBlocks: Record<string, string> = {
      build: `git clone https://github.com/quinnjr/spacey.git
cd spacey
cargo build --release
cargo run`,
      rust: `use spacey_spidermonkey::Engine;

fn main() {
    let mut engine = Engine::new();

    match engine.eval("1 + 2 * 3") {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}`,
      typescript: `// TypeScript runs natively!
interface User {
    name: string;
    age: number;
}

const greet = (user: User): string => {
    return \`Hello, \${user.name}!\`;
};

console.log(greet({ name: "World", age: 42 }));`,
      node: `const fs = require('node:fs');
const path = require('node:path');

// ES Modules also supported
import { readFile } from 'node:fs/promises';

async function main() {
    const data = await readFile('./data.json');
    console.log(data);
}`
    };

    const code = codeBlocks[section];
    if (code) {
      navigator.clipboard.writeText(code);
    }
  }
}


