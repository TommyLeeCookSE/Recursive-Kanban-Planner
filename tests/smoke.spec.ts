import { expect, test } from '@playwright/test';

test('smoke: app boots and core board actions are visible', async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.clear();
  });

  await page.goto('/');

  await expect(page.getByText('My Workspace')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Export' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Import' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Clear Cache' })).toBeVisible();

  await page.locator('main').getByRole('button', { name: '+ New Board' }).click();
  await page.getByPlaceholder('Enter title...').fill('Smoke Board');
  await page.getByRole('button', { name: 'Create Item' }).click();

  await expect(page.getByText('Smoke Board')).toBeVisible();
  await page.getByText('Smoke Board', { exact: true }).click();

  const main = page.locator('main');
  await expect(main.getByRole('button', { name: 'Create Bucket' })).toBeVisible();
  await expect(main.getByRole('button', { name: 'Open notes' })).toBeVisible();
  await expect(main.getByRole('button', { name: 'Labels' })).toBeVisible();
  await expect(main.getByRole('button', { name: 'Open settings' })).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(main.getByRole('button', { name: /Back to:/ })).toBeVisible();
  await expect(main.getByRole('button', { name: 'Create Bucket' })).toBeVisible();
  await expect(main.getByRole('button', { name: 'Open notes' })).toBeVisible();
  await expect(main.getByRole('button', { name: 'Labels' })).toBeVisible();
  await expect(main.getByRole('button', { name: 'Open settings' })).toBeVisible();
  await page.setViewportSize({ width: 1440, height: 900 });

  await main.getByRole('button', { name: 'Create Bucket' }).click();
  await page.getByPlaceholder('Column Name (e.g., Todo, Doing)').fill('Smoke Lane');
  await page.getByRole('button', { name: 'Add Column' }).click();

  await expect(page.getByRole('button', { name: 'Rename' })).toBeVisible();

  await page.getByRole('button', { name: '+' }).click();
  await page.getByPlaceholder('Enter title...').fill('Smoke Child');
  await page.getByRole('button', { name: 'Create Item' }).click();
  await expect(page.getByText('Smoke Child', { exact: true })).toBeVisible();

  await page.getByRole('button', { name: 'Rename' }).click();
  await page.getByRole('textbox').fill('Renamed Lane');
  await page.getByRole('button', { name: 'Save Changes' }).click();
  await expect(page.getByText('Renamed Lane')).toBeVisible();

  await page.getByRole('button', { name: /Back to:/ }).click();
  await expect(page.getByText('Smoke Lane')).toBeVisible();
  await expect(page.getByText('Smoke Child')).toBeVisible();

  const shell = page.locator('.app-shell');
  await expect(shell).toHaveClass(/theme-dark/);
  await page.getByRole('button', { name: /Evening|Sunrise/ }).click();
  await expect(shell).toHaveClass(/theme-light/);
});
