const { chromium } = require(process.env.EXPLICIT_PLAYWRIGHT_MODULE);
(async () => {
  const browser = await chromium.launch({headless: true, ...(process.env.EXPLICIT_BROWSER_EXECUTABLE ? {executablePath: process.env.EXPLICIT_BROWSER_EXECUTABLE} : {}), ...(process.env.EXPLICIT_BROWSER_CHANNEL ? {channel: process.env.EXPLICIT_BROWSER_CHANNEL} : {})});
  try {
    const context = await browser.newContext({javaScriptEnabled: false});
    const page = await context.newPage();
    await page.goto(process.env.EXPLICIT_BROWSER_URL);
    await page.getByRole('link', {name: 'Create account', exact: true}).click();
    await page.getByLabel('Username').fill('browser_user');
    await page.getByLabel('Password').fill('browser password twelve');
    await page.getByRole('button', {name: 'Create account', exact: true}).click();
    await page.getByLabel('New task').fill('<script>unsafe</script>');
    await page.getByRole('button', {name: 'Add task'}).click();
    await page.getByRole('link', {name: '<script>unsafe</script>', exact: true}).click();
    await page.getByLabel('Title').fill('Browser edited task');
    await page.getByLabel('Status').selectOption('true');
    await page.getByRole('button', {name: 'Save changes'}).click();
    if (await page.getByText('Complete', {exact: true}).count() !== 1) throw new Error('Completed state missing');
    await page.getByRole('link', {name: 'Browser edited task'}).click();
    await page.getByRole('button', {name: 'Delete task'}).click();
    if (await page.getByRole('link', {name: 'Browser edited task'}).count()) throw new Error('Task was not deleted');
    await page.getByRole('button', {name: 'Sign out'}).click();
    await page.getByRole('link', {name: 'Sign in', exact: true}).click();
    await page.getByLabel('Username').fill('browser_user');
    await page.getByLabel('Password').fill('browser password twelve');
    await page.getByRole('button', {name: 'Sign in', exact: true}).click();
    await page.getByRole('button', {name: 'Sign out'}).waitFor();
    console.log('PASS: real-browser registration, escaped task, edit, completion, deletion, logout and login');
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
