import { test, expect } from '@playwright/test';

test.describe('Minesweeper App', () => {
  test('should load the home page and show the title', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/Minesweeper/);
    await expect(page.locator('h1')).toContainText('Hello, world!');
  });

  test('should navigate to the Minesweeper board', async ({ page }) => {
    await page.goto('/');
    await page.click('text=Minesweeper');
    await expect(page).toHaveURL(/\/minesweeper/);
    
    // Check if the board is loading
    const board = page.locator('.board-container');
    
    // Wait for the board to appear
    await expect(board).toBeVisible({ timeout: 10000 });
    
    // Check for some game details that come from the backend
    await expect(page.locator('text=Game Id:')).toBeVisible();
    await expect(page.locator('text=Mines:')).toBeVisible();
  });

  test('should start a new game and interact with the board', async ({ page }) => {
    await page.goto('/minesweeper');
    
    // Wait for the game to load and board to be visible
    const board = page.locator('.board-container');
    await expect(board).toBeVisible({ timeout: 10000 });
    
    const initialGameIdText = await page.locator('small:has-text("Game Id:")').innerText();
    const initialGameId = initialGameIdText.split(': ')[1];
    
    // Click on a cell to reveal it. With CSS Grid, we can target cells directly.
    const cell = page.locator('.cell').first();
    await cell.click();
    
    // We expect some update to the board.
    await page.click('button:has-text("New Game")');
    
    // After clicking new game, the ID should change
    await expect(async () => {
      const newGameIdText = await page.locator('small:has-text("Game Id:")').innerText();
      const newGameId = newGameIdText.split(': ')[1];
      expect(newGameId).not.toBe(initialGameId);
    }).toPass();
  });
});
