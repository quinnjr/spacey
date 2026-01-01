import { Component, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';
import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import {
  faGithub,
  faNpm,
  faRust
} from '@fortawesome/free-brands-svg-icons';
import {
  faArrowRight,
  faBars,
  faXmark,
  faBox,
  faCube,
  faCubes,
  faGear,
  faServer,
  faCode,
  faWandMagicSparkles,
  faGlobe,
  faTerminal,
  faDownload,
  faCopy,
  faCheck,
  faArrowUpRightFromSquare
} from '@fortawesome/free-solid-svg-icons';

interface Crate {
  name: string;
  description: string;
  longDescription: string;
  icon: any;
  color: string;
  features: string[];
  dependencies: string[];
  usage: string;
  cratesIoUrl: string;
  docsRsUrl: string;
}

@Component({
  selector: 'app-crates',
  standalone: true,
  imports: [CommonModule, RouterModule, FontAwesomeModule],
  templateUrl: './crates.html',
  styleUrl: './crates.css'
})
export class CratesComponent {
  // Icons
  faGithub = faGithub;
  faNpm = faNpm;
  faRust = faRust;
  faArrowRight = faArrowRight;
  faBars = faBars;
  faXmark = faXmark;
  faBox = faBox;
  faCube = faCube;
  faCubes = faCubes;
  faGear = faGear;
  faServer = faServer;
  faCode = faCode;
  faWandMagicSparkles = faWandMagicSparkles;
  faGlobe = faGlobe;
  faTerminal = faTerminal;
  faDownload = faDownload;
  faCopy = faCopy;
  faCheck = faCheck;
  faArrowUpRightFromSquare = faArrowUpRightFromSquare;

  mobileMenuOpen = signal(false);
  copiedCrate = signal<string | null>(null);

  crates: Crate[] = [
    {
      name: 'spacey',
      description: 'JavaScript engine and REPL',
      longDescription: 'The main entry point for Spacey. Provides a command-line REPL for interactive JavaScript/TypeScript execution and serves as the reference implementation for the engine.',
      icon: this.faTerminal,
      color: '#ff7139',
      features: [
        'Interactive REPL with syntax highlighting',
        'Command history and completion',
        'Execute .js and .ts files directly',
        'Colorful error messages'
      ],
      dependencies: ['spacey-spidermonkey'],
      usage: `cargo install spacey

# Run the REPL
spacey

# Execute a file
spacey run script.js`,
      cratesIoUrl: 'https://crates.io/crates/spacey',
      docsRsUrl: 'https://docs.rs/spacey'
    },
    {
      name: 'spacey-spidermonkey',
      description: 'Core JavaScript engine',
      longDescription: 'The heart of Spacey. A complete JavaScript engine written in pure Rust, inspired by Mozilla\'s SpiderMonkey. Includes lexer, parser, compiler, and virtual machine.',
      icon: this.faGear,
      color: '#9059ff',
      features: [
        'Full ES3 specification compliance',
        'Native TypeScript parsing',
        'Bytecode compiler & VM',
        'Generational garbage collector',
        'Arena-based AST allocation'
      ],
      dependencies: [],
      usage: `use spacey_spidermonkey::Engine;

fn main() {
    let mut engine = Engine::new();

    match engine.eval("1 + 2 * 3") {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}`,
      cratesIoUrl: 'https://crates.io/crates/spacey-spidermonkey',
      docsRsUrl: 'https://docs.rs/spacey-spidermonkey'
    },
    {
      name: 'spacey-macros',
      description: 'Developer-friendly utility macros',
      longDescription: 'A collection of macros that enhance the developer experience when working with Spacey. Includes timing utilities, error handling, collections, and more.',
      icon: this.faWandMagicSparkles,
      color: '#10b981',
      features: [
        'js_object! - Create JS objects easily',
        'js_array! - Create JS arrays',
        'time_it! - Performance measurement',
        'defer! - Go-style defer statements',
        'int_enum! - Integer enums with conversion'
      ],
      dependencies: [],
      usage: `use spacey_macros::{js_object, time_it, defer};

// Create a JS object
let obj = js_object! {
    "name" => "Spacey",
    "version" => 1
};

// Measure execution time
time_it!("parsing", {
    parser.parse(source)?;
});

// Defer cleanup
defer!(println!("cleanup"));`,
      cratesIoUrl: 'https://crates.io/crates/spacey-macros',
      docsRsUrl: 'https://docs.rs/spacey-macros'
    },
    {
      name: 'spacey-servo',
      description: 'Servo browser integration',
      longDescription: 'Integration layer for using Spacey as the JavaScript engine in the Servo web browser. Provides DOM bindings, event loop integration, and browser-specific APIs.',
      icon: this.faGlobe,
      color: '#3b82f6',
      features: [
        'DOM API bindings (Window, Document, Element)',
        'Event loop with microtasks/macrotasks',
        'Console API implementation',
        'Script execution context',
        'Ready for Servo integration'
      ],
      dependencies: ['spacey-spidermonkey', 'spacey-macros'],
      usage: `use spacey_servo::SpaceyServo;

fn main() {
    let servo = SpaceyServo::new();

    // Execute script in browser context
    servo.execute_script(r#"
        document.getElementById('app')
            .textContent = 'Hello from Spacey!';
    "#)?;
}`,
      cratesIoUrl: 'https://crates.io/crates/spacey-servo',
      docsRsUrl: 'https://docs.rs/spacey-servo'
    },
    {
      name: 'spacey-node',
      description: 'Node.js-compatible runtime',
      longDescription: 'A Node.js-compatible runtime built on Spacey. Run your Node.js applications with a pure Rust JavaScript engine. Supports CommonJS, ES Modules, and core Node.js APIs.',
      icon: this.faServer,
      color: '#f59e0b',
      features: [
        'CommonJS require()',
        'ES Module import/export',
        'node:fs, node:path, node:crypto',
        'node:http server',
        'Async I/O via Tokio',
        'npm package resolution'
      ],
      dependencies: ['spacey-spidermonkey'],
      usage: `cargo install spacey-node

# Run a Node.js script
spacey-node server.js

# Start REPL with Node.js APIs
spacey-node`,
      cratesIoUrl: 'https://crates.io/crates/spacey-node',
      docsRsUrl: 'https://docs.rs/spacey-node'
    },
    {
      name: 'spacey-npm',
      description: 'Fast, async NPM package manager',
      longDescription: 'A blazing-fast, multi-threaded npm-compatible package manager written in Rust. Downloads packages in parallel, uses efficient caching, and provides a snappy CLI.',
      icon: this.faBox,
      color: '#ec4899',
      features: [
        'Parallel downloads',
        'Lock file support',
        'npm registry compatible',
        'Workspace support',
        'Offline caching',
        'Semantic version resolution'
      ],
      dependencies: [],
      usage: `cargo install spacey-npm

# Install dependencies
snpm install

# Add a package
snpm add lodash

# Run a script
snpm run build`,
      cratesIoUrl: 'https://crates.io/crates/spacey-npm',
      docsRsUrl: 'https://docs.rs/spacey-npm'
    }
  ];

  // Stars for background animation
  stars = Array.from({ length: 50 }, (_, i) => ({
    id: i,
    x: Math.random() * 100,
    y: Math.random() * 100,
    delay: Math.random() * 3,
    opacity: 0.3 + Math.random() * 0.5
  }));

  toggleMobileMenu() {
    this.mobileMenuOpen.update(v => !v);
  }

  copyInstall(crateName: string) {
    const command = crateName === 'spacey-npm'
      ? `cargo install spacey-npm`
      : `cargo add ${crateName}`;

    navigator.clipboard.writeText(command);
    this.copiedCrate.set(crateName);

    setTimeout(() => {
      this.copiedCrate.set(null);
    }, 2000);
  }

  copyCode(code: string) {
    navigator.clipboard.writeText(code);
  }
}
