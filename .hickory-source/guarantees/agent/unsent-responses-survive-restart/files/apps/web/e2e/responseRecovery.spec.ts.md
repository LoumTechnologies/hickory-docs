<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/e2e/responseRecovery.spec.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Unsent responses survive restart](../../../../contract.md)

Captured SHA-256: `826ade25bb8f77b6693c51a86006553f2fd4d8f8ead7c60cb7b5961abcaf4d73`.

<book:entry path="apps/web/e2e/responseRecovery.spec.ts">
// Guarantee: docs/guarantees/agent/unsent-responses-survive-restart.md
import { expect, test } from "@playwright/test";

test('unsent response survives a fresh browser page and explicit discard removes it', async ({ page, context }) => {
  test.skip(!process.env.HICKORY_SPATIAL_URL, 'Use just test-response-recovery-ui.');
  await page.goto(process.env.HICKORY_SPATIAL_URL!);
  await expect(page.getByRole('tablist', { name: 'Center views' })).toBeVisible();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent('hickory-menu', { detail: 'show-agent' })));
  const agent = page.getByRole('region', { name: 'Agent chat' });
  const response = agent.getByRole('textbox', { name: 'Agent conversation and response' });
  await response.click(); await response.press('ControlOrMeta+End');
  await response.pressSequentially('Durable unsent response');
  // Observe the real server's persisted bytes, independently of the editor.
  await expect.poll(async () => (await (await page.request.get('/api/workspace/responses?doc=workspace')).json()).state?.drafts.new).toBe('Durable unsent response');
  const old = await (await page.request.get('/api/workspace/responses?doc=workspace')).json();
  expect((await page.request.put('/api/workspace/responses?doc=workspace', { data: { scope: 'another workspace', state: old.state } })).status()).toBe(409);
  const url = page.url();
  await page.close();
  const fresh = await context.newPage(); await fresh.goto(url);
  await fresh.evaluate(() => window.dispatchEvent(new CustomEvent('hickory-menu', { detail: 'show-agent' })));
  const reopened = fresh.getByRole('region', { name: 'Agent chat' });
  await expect(reopened.getByRole('textbox', { name: 'Agent conversation and response' })).toContainText('Durable unsent response');
  await reopened.getByRole('button', { name: 'Discard response', exact: true }).click();
  await expect.poll(async () => (await (await fresh.request.get('/api/workspace/responses?doc=workspace')).json()).state?.drafts.new).toBe('');
  await fresh.reload();
  await fresh.evaluate(() => window.dispatchEvent(new CustomEvent('hickory-menu', { detail: 'show-agent' })));
  await expect(fresh.getByRole('textbox', { name: 'Agent conversation and response' })).not.toContainText('Durable unsent response');
  // Recovery never writes its text into document drafts or session evidence.
  const drafts = await (await fresh.request.get('/api/workspace/drafts')).json();
  expect(JSON.stringify(drafts)).not.toContain('Durable unsent response');
  const turns = await (await fresh.request.get('/api/docs/workspace/agent/turns')).json();
  expect(turns.turns).toHaveLength(0);
});

test('existing thread restores when durable storage arrives before turn history', async ({ page }) => {
  test.skip(!process.env.HICKORY_SPATIAL_URL, 'Use just test-response-recovery-ui.');
  await page.goto(process.env.HICKORY_SPATIAL_URL!);
  const stored = await (await page.request.get('/api/workspace/responses?doc=workspace')).json();
  expect((await page.request.put('/api/workspace/responses?doc=workspace', { data: { scope: stored.scope,
    state: { version: 1, tip: 'child', drafts: { root: 'Reply to existing thread' } } } })).ok()).toBeTruthy();
  await page.route('**/api/docs/workspace/agent/turns', async route => {
    await new Promise(resolve => setTimeout(resolve, 400));
    const turn = (id: string, parent_id: string | null) => ({ id, parent_id, prompt: id, answer: 'answer', status: 'ok', error: null,
      created_at: '2026-10-17T00:00:00Z', provider: 'anthropic', model: 'test', usage: null });
    await route.fulfill({ json: { turns: [turn('root', null), turn('child', 'root')], provider: 'anthropic', model: 'test',
      totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 } } });
  });
  await page.reload();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent('hickory-menu', { detail: 'show-agent' })));
  await expect(page.getByRole('textbox', { name: 'Agent conversation and response' })).toContainText('Reply to existing thread');
});
</book:entry>

</book:doc>
