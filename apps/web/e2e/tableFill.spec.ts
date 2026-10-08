// Guarantee: docs/guarantees/authoring/a-fill-handle-extends-formulas.md
import { expect, test } from '@playwright/test';
for (const [row,column,formula] of [[0,2,'=B1*2'],[4,2,'=B5*2'],[2,0,'=#REF!*2'],[2,4,'=D3*2']] as const) {
  test(`drag the fill handle to ${row},${column}`,async({page})=>{
    test.skip(!process.env.HICKORY_E2E_URL,'Run against the full editor.');
    await page.goto('/');
    const pane=page.locator('.untitled-tab');
    await pane.locator('.cm-content').click();
    await page.keyboard.press('ControlOrMeta+a');
    await page.keyboard.insertText('<hick:table language="javascript" header="false" formats="C3:USD">\n1,2,3,4,5\n6,7,8,9,10\n11,12,=B3*2,14,15\n16,17,18,19,20\n21,22,23,24,25\n</hick:table>');
    await pane.getByRole('button',{name:/Table 1.*click to render/}).click();
    const cell=(r:number,c:number)=>pane.locator(`.table-panel__cell[data-row="${r}"][data-column="${c}"]`);
    await cell(2,2).click();
    const handle=pane.getByRole('button',{name:'Fill selected cells'});
    const start=(await handle.boundingBox())!;
    const end=(await cell(row,column).boundingBox())!;
    await page.mouse.move(start.x+start.width/2,start.y+start.height/2);
    await page.mouse.down();
    await page.mouse.move(end.x+end.width/2,end.y+end.height/2,{steps:8});
    await expect(pane.locator('.table-fill-preview').first()).toBeVisible();
    await page.mouse.up();
    await cell(row,column).click();
    await expect(pane.locator('.table-panel__formula')).toHaveValue(formula);
    await expect(pane.getByLabel('Cell number format')).toHaveValue('USD');
  });
}
test('fills a selected formula block beyond the bottom edge',async({page})=>{
  test.skip(!process.env.HICKORY_E2E_URL,'Run against the full editor.');
  await page.goto('/');
  const pane=page.locator('.untitled-tab');
  await pane.locator('.cm-content').click();
  await page.keyboard.press('ControlOrMeta+a');
  await page.keyboard.insertText('<hick:table language="python" header="false">\n1,=A1*2\n2,=A2*3\n3,\n4,\n</hick:table>');
  await pane.getByRole('button',{name:/Table 1.*click to render/}).click();
  const cell=(row:number)=>pane.locator(`.table-panel__cell[data-row="${row}"][data-column="1"]`);
  await cell(1).click();
  await cell(0).click({modifiers:['Shift']});
  const handle=pane.getByRole('button',{name:'Fill selected cells'});
  await expect(handle).toHaveCount(1);
  const start=(await handle.boundingBox())!;
  const last=(await cell(3).boundingBox())!;
  await page.mouse.move(start.x+start.width/2,start.y+start.height/2);
  await page.mouse.down();
  await page.mouse.move(last.x+last.width/2,last.y+last.height*2.5,{steps:8});
  await expect(pane.locator('.table-fill-range')).toContainText('B1:B6');
  await page.mouse.up();
  await cell(4).click();
  await expect(pane.locator('.table-panel__formula')).toHaveValue('=A5*2');
  await cell(5).click();
  await expect(pane.locator('.table-panel__formula')).toHaveValue('=A6*3');
});
