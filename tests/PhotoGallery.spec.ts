import { test, expect } from '@playwright/experimental-ct-vue';
import PhotoGallery from '@/components/PhotoGallery.vue';

test.use({ viewport: { width: 1000, height: 800 } });

test('render empty state', async ({ mount, page }) => {
    await page.evaluate(() => (window as any).__MOCK_PHOTOS__ = { photos: [] });

    const component = await mount(PhotoGallery, {
        props: {
            photos: [], // prop is reactive but mock composable also holds state
            loading: false
        }
    });
    await expect(component).toContainText('No photos found');
});

test('render photos', async ({ mount, page }) => {
    const mockPhotos = ['https://via.placeholder.com/300', 'https://via.placeholder.com/301'];

    // Inject into mock composable state
    await page.evaluate((photos) => {
        (window as any).__MOCK_PHOTOS__ = { photos };
    }, mockPhotos);

    const component = await mount(PhotoGallery, {
        props: {
            photos: mockPhotos,
            loading: false
        }
    });

    // Check if images are rendered
    const images = component.locator('img');
    await expect(images).toHaveCount(2);
});
