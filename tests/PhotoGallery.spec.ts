/**
 * TypeScript Module - 0 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { test, expect } from '@playwright/experimental-ct-vue';
import PhotoGallery from '@/components/PhotoGallery.vue';

test.use({ viewport: { width: 1000, height: 800 } });

test('render empty state', async ({ mount, page }) => {
    await page.evaluate(() => (window as any).__MOCK_PHOTOS__ = { photos: [] });

    const component = await mount(PhotoGallery, {
        props: {
            photos: [], 
            loading: false
        }
    });
    await expect(component).toContainText('No photos found');
});

test('render photos', async ({ mount, page }) => {
    const mockPhotos = ['https:

    await page.evaluate((photos) => {
        (window as any).__MOCK_PHOTOS__ = { photos };
    }, mockPhotos);

    const component = await mount(PhotoGallery, {
        props: {
            photos: mockPhotos,
            loading: false
        }
    });

    const images = component.locator('img');
    await expect(images).toHaveCount(2);
});