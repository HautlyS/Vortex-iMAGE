import { defineConfig, devices } from '@playwright/experimental-ct-vue';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export default defineConfig({
    testDir: './tests',
    snapshotDir: './__snapshots__',
    timeout: 10 * 1000,
    fullyParallel: true,
    forbidOnly: !!process.env.CI,
    retries: process.env.CI ? 2 : 0,
    workers: process.env.CI ? 1 : undefined,
    reporter: 'html',
    use: {
        trace: 'on-first-retry',
        ctPort: 3100,
        ctViteConfig: {
            resolve: {
                alias: {
                    'vue': resolve(__dirname, 'node_modules/vue'),
                    '@': resolve(__dirname, './src'),
                    [resolve(__dirname, './src/composables/useGitHubAuth.ts')]: resolve(__dirname, './tests/mocks/useGitHubAuth.ts'),
                    [resolve(__dirname, './src/composables/usePhotoUpload.ts')]: resolve(__dirname, './tests/mocks/usePhotoUpload.ts'),
                    [resolve(__dirname, './src/composables/useAccentColor.ts')]: resolve(__dirname, './tests/mocks/useAccentColor.ts'),
                }
            }
        }
    },
    projects: [
        {
            name: 'chromium',
            use: { ...devices['Desktop Chrome'] },
        },
        {
            name: 'firefox',
            use: { ...devices['Desktop Firefox'] },
        },
        {
            name: 'webkit',
            use: { ...devices['Desktop Safari'] },
        },
    ],
});
