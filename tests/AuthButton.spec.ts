import { test, expect } from '@playwright/experimental-ct-vue';
import AuthButton from '@/components/AuthButton.vue';

test.use({ viewport: { width: 500, height: 500 } });

test('render login button when not authenticated', async ({ mount, page }) => {
    const component = await mount(AuthButton);
    // Ensure logged out
    await page.evaluate(() => (window as any).__SET_MOCK_USER__?.(null));
    await expect(component).toContainText('Sign In with GitHub');
});

test('render user profile when authenticated', async ({ mount, page }) => {
    const component = await mount(AuthButton);

    // Trigger reactive update
    await page.evaluate(() => {
        (window as any).__SET_MOCK_USER__?.({
            login: 'test-user',
            avatar_url: 'https://via.placeholder.com/150'
        });
    });

    await expect(component).toContainText('test-user');
    await expect(component.locator('img')).toBeVisible();
});

test('render device code when code is present', async ({ mount, page }) => {
    const component = await mount(AuthButton);

    await page.evaluate(() => {
        (window as any).__SET_MOCK_USER__?.(null);
        (window as any).__SET_MOCK_CODE__?.('1234-5678');
    });

    await expect(component).toContainText('1234-5678');
});
