# Spacey Browser Internal UI

This Angular application provides the internal pages for Spacey Browser, including:

- **Welcome Page** (`/welcome`) - Browser welcome and quick start
- **Settings** (`/settings`) - Browser configuration
- **Bug Report** (`/bugreport`) - Submit bug reports via FormSubmit to support@pegasusheavy.dev
- **Thank You** (`/bugreport-thanks`) - Confirmation after bug submission

## Development

```bash
# Install dependencies
npm install

# Start development server
npm start
```

The app will be available at http://localhost:4200

## Building

```bash
npm run build
```

This outputs to `../../assets/ui/` which is picked up by the Rust browser binary.

## Browser Bridge

The app communicates with the Rust backend via `window.spaceyBridge`:

```typescript
interface SpaceyBridge {
  getSystemInfo(): SystemInfo;
  setShieldLevel(level: string): void;
  saveSettings(settings: any): void;
}
```

In development mode, the app uses fallback values and localStorage.

## Routing

The browser maps `about:` URLs to Angular routes:

| Browser URL | Angular Route |
|-------------|---------------|
| `about:welcome` | `/welcome` |
| `about:settings` | `/settings` |
| `about:bugreport` | `/bugreport` |
| `about:bugreport-thanks` | `/bugreport-thanks` |
