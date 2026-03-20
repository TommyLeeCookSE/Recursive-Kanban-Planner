import { expect, test } from '@playwright/test';

test('smoke: app boots and core board actions are visible', async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.clear();
  });

  await page.goto('/');

  await expect(page.getByRole('heading', { name: 'My Workspace' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Export' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Import' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Clear Cache' })).toBeVisible();
  await expect(page.getByTitle('Back to My Workspace')).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Create Card' })).toBeVisible();

  await page.locator('main').getByRole('button', { name: 'Create Card' }).click();
  await page.getByPlaceholder('Enter title...').fill('Smoke Board');
  await page.getByTitle('Create this card').click();

  await expect(page.getByText('Smoke Board')).toBeVisible();
  await page.getByText('Smoke Board', { exact: true }).click();

  const main = page.locator('main');
  await expect(main.getByRole('button', { name: 'Create Card' })).toBeVisible();
  await expect(main.getByRole('button', { name: 'Notes' })).toBeVisible();
  await expect(main.getByRole('button', { name: 'Settings' })).toBeVisible();
  await expect(main.getByRole('button', { name: 'Create Bucket' })).toHaveCount(0);

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(main.getByTitle('Back to My Workspace')).toBeVisible();
  await expect(main.getByRole('button', { name: 'Create Card' })).toBeVisible();
  await expect(main.getByRole('button', { name: 'Notes' })).toBeVisible();
  await expect(main.getByRole('button', { name: 'Settings' })).toBeVisible();
  await page.setViewportSize({ width: 1440, height: 900 });

  await main.getByRole('button', { name: 'Create Card' }).click();
  await page.getByPlaceholder('Enter title...').fill('Smoke Child');
  await page.getByTitle('Create this card').click();
  await expect(page.getByText('Smoke Child', { exact: true })).toBeVisible();

  await page.getByTitle('Back to My Workspace').click();
  await expect(page.getByText('Smoke Board', { exact: true })).toBeVisible();
  await expect(page.getByText('Smoke Child', { exact: true })).toBeVisible();

  const shell = page.locator('.app-shell');
  await expect(shell).toHaveClass(/theme-dark/);
  await page.getByRole('button', { name: /Switch to|Evening|Sunrise/ }).click();
  await expect(shell).toHaveClass(/theme-light/);
});
