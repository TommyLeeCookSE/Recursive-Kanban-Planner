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
  await expect(main.getByRole('button', { name: 'Notes' })).toBeVisible();
  await expect(main.getByRole('button', { name: 'Labels' })).toBeVisible();

  await main.getByRole('button', { name: 'Create Bucket' }).click();
  await page.getByPlaceholder('Column Name (e.g., Todo, Doing)').fill('Smoke Lane');
  await page.getByRole('button', { name: 'Add Column' }).click();

  await expect(page.getByRole('button', { name: 'Rename' })).toBeVisible();

  await page.getByRole('button', { name: 'Rename' }).click();
  await page.getByRole('textbox').fill('Renamed Lane');
  await page.getByRole('button', { name: 'Save Changes' }).click();
  await expect(page.getByText('Renamed Lane')).toBeVisible();

  const shell = page.locator('.app-shell');
  await expect(shell).toHaveClass(/theme-dark/);
  await page.getByRole('button', { name: /Evening|Sunrise/ }).click();
  await expect(shell).toHaveClass(/theme-light/);
});
