// Run against an isolated wing-server and the web frontend with Playwright installed.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { chromium } = require('playwright');

(async () => {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), 'wingdrive-explorer-'));
  const output = process.env.WINGDRIVE_TEST_OUTPUT || '/tmp/wingdrive-explorer-results';
  fs.mkdirSync(output, {recursive: true});
  fs.mkdirSync(path.join(fixture, 'folder-A'));
  fs.mkdirSync(path.join(fixture, 'folder-B'));
  for (let i = 0; i < 10000; i++) {
    const file = path.join(fixture, `item-${String(i).padStart(5, '0')}.txt`);
    fs.writeFileSync(file, Buffer.alloc(i % 101, 120));
    fs.utimesSync(file, 1700000000 + i, 1700000000 + i);
  }
  fs.copyFileSync('apps/tauri/src-tauri/icons/128x128.png', path.join(fixture, 'wing.png'));
  const browser = await chromium.launch({headless: true,
    ...(process.env.CHROMIUM_PATH ? {executablePath: process.env.CHROMIUM_PATH} : {}),
    args: ['--no-sandbox']});
  const page = await browser.newPage({viewport: {width: 1400, height: 900}});
  page.setDefaultTimeout(20000);
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  if (process.env.RPC_URL) {
    await page.route('**/rpc', async route => {
      const response = await route.fetch({url: process.env.RPC_URL});
      await route.fulfill({response});
    });
  }
  const urlFor = dir => '/explorer?path=' + encodeURIComponent(JSON.stringify({Physical: {device_slug: 'local', path: dir}}));
  const location = urlFor(fixture);
  const perfDirectory = path.join(fixture, 'folder-B');
  const dropDirectory = path.join(perfDirectory, 'drop-target');
  fs.mkdirSync(dropDirectory);
  for (let i = 0; i < 50; i++) fs.writeFileSync(path.join(perfDirectory, `perf-${i}.txt`), 'WingDrive');
  const base = process.env.BASE_URL || 'http://127.0.0.1:3001';
  const files = page.locator('[data-file-id]');
  const selected = page.locator('[data-file-id][aria-selected=true]');
  const mode = async name => {
    await page.getByRole('button', {name: 'Views', exact: true}).click();
    await page.getByRole('button', {name: new RegExp(`^${name}`)}).click();
  };
  const scroll = async bottom => page.evaluate(bottom => {
    let element = document.querySelector('[data-file-id]');
    while (element && !(element.scrollHeight > element.clientHeight && getComputedStyle(element).overflowY === 'auto')) element = element.parentElement;
    if (!element) throw Error('No explorer scroll container');
    element.scrollTop = bottom ? element.scrollHeight : 0;
    return {top: element.scrollTop, height: element.scrollHeight};
  }, bottom);
  try {
    console.log('Grid');
    await page.goto(base + location);
    await page.getByText('item-00000.txt', {exact: true}).waitFor({timeout: 90000});
    assert((await files.count()) < 300, 'Grid mounted all files');
    await page.getByText('item-00001.txt', {exact: true}).click();
    await page.getByText('item-00003.txt', {exact: true}).click({modifiers: ['Control']});
    assert.equal(await selected.count(), 2, 'Ctrl selection');
    await page.getByText('item-00005.txt', {exact: true}).click({modifiers: ['Shift']});
    assert((await selected.count()) >= 3, 'Shift range selection');
    await page.keyboard.press('ArrowDown');
    assert.equal(await selected.count(), 1, 'Grid keyboard selection');
    const gridScroll = await scroll(true);
    await page.getByText('item-09999.txt', {exact: true}).waitFor();
    assert(gridScroll.height > 100000, '10k grid did not load');
    assert((await files.count()) < 300, 'Grid virtualization after scroll');
    const gridMounted = await files.count();
    await page.locator('[data-file-id]').filter({hasText: 'wing.png'}).locator('img').first().waitFor();
    await page.waitForFunction(() => Array.from(document.querySelectorAll('[data-file-id] img')).some(image => image.complete && image.naturalWidth > 0));
    await scroll(false);
    await page.getByText('item-00000.txt', {exact: true}).waitFor();
    await page.screenshot({path: path.join(output, 'WingGridView.png')});
    console.log('List');
    await mode('List');
    for (const name of ['Name', 'Size', 'Modified', 'Kind', 'Tags']) await page.getByRole('columnheader', {name, exact: true}).waitFor();
    const size = page.getByRole('columnheader', {name: 'Size', exact: true});
    await size.click();
    assert.equal(await size.getAttribute('aria-sort'), 'descending');
    await size.click();
    assert.equal(await size.getAttribute('aria-sort'), 'ascending');
    await page.getByRole('columnheader', {name: 'Modified', exact: true}).click({modifiers: ['Shift']});
    assert.equal(await size.getAttribute('aria-sort'), 'ascending', 'Shift sorting lost primary column');
    assert.notEqual(await page.getByRole('columnheader', {name: 'Modified', exact: true}).getAttribute('aria-sort'), 'none');
    const name = page.getByRole('columnheader', {name: 'Name', exact: true});
    const before = await name.boundingBox();
    await page.mouse.move(before.x + before.width - 2, before.y + before.height / 2);
    await page.mouse.down();
    await page.mouse.move(before.x + before.width + 58, before.y + before.height / 2, {steps: 10});
    await page.mouse.up();
    assert((await name.boundingBox()).width > before.width + 40, 'Column resize');
    await page.keyboard.press('ArrowDown');
    assert.equal(await selected.count(), 1, 'List keyboard selection');
    assert((await files.count()) < 150, 'List mounted all files');
    await page.screenshot({path: path.join(output, 'WingListView.png')});
    await mode('Grid');
    console.log('History');
    await page.locator('[data-file-id]').getByText('folder-A', {exact: true}).dblclick();
    await page.waitForURL('**' + encodeURIComponent('folder-A') + '**');
    await page.keyboard.press('Control+ArrowLeft');
    await page.locator('[data-file-id]').getByText('folder-A', {exact: true}).waitFor();
    console.log("Back", page.url());
    const saved = await page.evaluate(() => JSON.parse(localStorage.getItem('wing-tabs-state')));
    assert(saved.tabs[0].history.length >= 2, 'Tab history missing');
    const originalId = saved.activeTabId;
    console.log('New/switch tab');
    await page.getByRole('button', {name: 'New tab (Ctrl+T)', exact: true}).click();
    assert.notEqual(page.url(), base + location, 'New tab route unchanged');
    await page.locator(`[data-tab-id="${originalId}"] button`).first().click();
    await page.getByText('item-00000.txt', {exact: true}).waitFor();
    assert.equal(await selected.count(), 1, 'Tab selection was lost');
    await scroll(true);
    await page.getByText('item-09999.txt', {exact: true}).waitFor();
    console.log('New/switch tab');
    await page.getByRole('button', {name: 'New tab (Ctrl+T)', exact: true}).click();
    await page.locator(`[data-tab-id="${originalId}"] button`).first().click();
    await page.getByText('item-09999.txt', {exact: true}).waitFor();
    assert(!await page.getByText('item-00000.txt', {exact: true}).count(), 'Tab scroll was lost');
    await scroll(false);
    console.log('Column');
    await mode('Column');
    await page.locator('[data-file-id]').getByText('folder-A', {exact: true}).waitFor();
    await page.screenshot({path: path.join(output, 'WingColumnView.png')});
    console.log('Media');
    await mode('Media');
    await page.screenshot({path: path.join(output, 'WingMediaView.png')});
    console.log('Size');
    await mode('Size');
    await page.screenshot({path: path.join(output, 'WingSizeView.png')});
    await mode('Grid');
    await page.getByText('item-00000.txt', {exact: true}).waitFor();
    console.log('Cross-tab drag');
    await page.evaluate(locations => {
      const state = JSON.parse(localStorage.getItem('wing-tabs-state'));
      const first = state.tabs.find(tab => tab.id === state.activeTabId);
      const prefs = state.explorerStates[first.id];
      state.tabs = locations.map((location, i) => ({...first, id: `drag-${i}`, title: `Drag ${i}`, savedPath: location, history: [location], historyIndex: 0}));
      state.activeTabId = 'drag-0';
      state.explorerStates = Object.fromEntries(state.tabs.map(tab => [tab.id, {...prefs, viewMode: 'grid', scrollTop: 0}]));
      localStorage.setItem('wing-tabs-state', JSON.stringify(state));
    }, [location, urlFor(perfDirectory)]);
    await page.reload();
    const dragFile = page.locator('[data-file-id]').filter({hasText: 'item-00000.txt'});
    await dragFile.waitFor();
    const dragBox = await dragFile.boundingBox();
    const tabBox = await page.locator('[data-tab-id="drag-1"]').boundingBox();
    await page.mouse.move(dragBox.x + dragBox.width / 2, dragBox.y + 30);
    await page.mouse.down();
    await page.mouse.move(tabBox.x + tabBox.width / 2, tabBox.y + tabBox.height / 2, {steps: 15});
    await page.locator('[data-tab-id="drag-1"][data-active=true]').waitFor();
    const dropFolder = page.locator('[data-file-id]').filter({hasText: 'drop-target'});
    await dropFolder.waitFor();
    const dropBox = await dropFolder.boundingBox();
    await page.mouse.move(dropBox.x + dropBox.width / 2, dropBox.y + 30, {steps: 10});
    await page.mouse.up();
    await page.getByRole('dialog').getByRole('button', {name: 'Move', exact: true}).last().click();
    await page.waitForFunction(() => !document.querySelector('[role="dialog"]'));
    assert(fs.existsSync(path.join(dropDirectory, 'item-00000.txt')), 'Cross-tab move missing');
    assert(!fs.existsSync(path.join(fixture, 'item-00000.txt')), 'Cross-tab move kept source');
    console.log('Close and reopen tabs');
    await page.keyboard.press('Control+w');
    await page.locator('[data-tab-id="drag-1"]').waitFor({state: 'detached'});
    await page.keyboard.press('Control+w');
    assert.equal(await page.locator('[data-tab-id]').count(), 1, 'Last tab closed');
    await page.keyboard.press('Control+Shift+t');
    await page.locator('[data-tab-id="drag-1"][data-active=true]').waitFor();
    console.log('Performance');
    const cdp = await page.context().newCDPSession(page);
    const browserCdp = await browser.newBrowserCDPSession();
    const browserMemory = async () => {
      const {processInfo} = await browserCdp.send('SystemInfo.getProcessInfo');
      return processInfo.reduce((sum, process) => {
        const status = fs.readFileSync(`/proc/${process.id}/smaps_rollup`, 'utf8');
        const memory = Number(status.match(/^Pss:\s+(\d+)/m)[1]) * 1024;
        return {total: sum.total + memory, renderer: sum.renderer + (process.type === 'renderer' ? memory : 0)};
      }, {total: 0, renderer: 0});
    };
    const perfLocation = urlFor(perfDirectory);
    await page.evaluate(location => {
      const state = JSON.parse(localStorage.getItem('wing-tabs-state'));
      const first = state.tabs.find(tab => tab.id === state.activeTabId);
      const prefs = state.explorerStates[first.id];
      state.tabs = Array.from({length: 15}, (_, i) => ({...first, id: `perf-${i}`, title: `Tab ${i + 1}`, savedPath: location, history: [location], historyIndex: 0}));
      state.activeTabId = 'perf-0';
      state.explorerStates = Object.fromEntries(state.tabs.map(tab => [tab.id, {...prefs, viewMode: 'grid', scrollTop: 0}]));
      localStorage.setItem('wing-tabs-state', JSON.stringify(state));
    }, perfLocation);
    await page.goto(base + perfLocation);
    console.log('Reloading 15 tabs');
    await page.reload();
    await page.getByText('perf-0.txt', {exact: true}).waitFor();
    await cdp.send('HeapProfiler.collectGarbage');
    const memoryBefore = (await cdp.send('Runtime.getHeapUsage')).usedSize;
    const browserMemoryBefore = await browserMemory();
    const timings = [];
    for (let i = 1; i <= 100; i++) {
      if (i % 10 === 0) console.log('Cycle', i);
      timings.push(await page.evaluate(id => new Promise(resolve => {
        const start = performance.now();
        document.querySelector(`[data-tab-id="${id}"] button`).click();
        requestAnimationFrame(() => resolve(performance.now() - start));
      }), `perf-${i % 15}`));
      await page.getByText('perf-0.txt', {exact: true}).waitFor();
    }
    await cdp.send('HeapProfiler.collectGarbage');
    const memoryAfter = (await cdp.send('Runtime.getHeapUsage')).usedSize;
    const browserMemoryAfter = await browserMemory();
    console.log('Memory', {heapBefore: memoryBefore/1048576, heapAfter: memoryAfter/1048576,
      browserBefore: browserMemoryBefore.total/1048576, browserAfter: browserMemoryAfter.total/1048576,
      rendererBefore: browserMemoryBefore.renderer/1048576, rendererAfter: browserMemoryAfter.renderer/1048576,
      switchMedian: timings.slice().sort((a,b)=>a-b)[50]});
    assert(browserMemoryAfter.renderer < 500 * 1024 * 1024, '15 tabs exceeded 500 MB renderer memory');
    assert(memoryAfter - memoryBefore < 100 * 1024 * 1024, '100 tab switches retained excessive memory');
    console.log('Missing location');
    await page.evaluate(location => {
      const state = JSON.parse(localStorage.getItem('wing-tabs-state'));
      const tab = state.tabs.find(tab => tab.id === state.activeTabId);
      tab.savedPath = location;
      tab.history = [location];
      tab.historyIndex = 0;
      localStorage.setItem('wing-tabs-state', JSON.stringify(state));
    }, urlFor(path.join(fixture, 'deleted-location')));
    await page.goto(base + urlFor(path.join(fixture, 'deleted-location')));
    await page.getByRole('alert').filter({hasText: 'Could not open this location'}).waitFor();
    await page.getByRole('button', {name: 'Go to Overview', exact: true}).click();
    await page.waitForURL(base + '/');
    assert.equal(errors.length, 0, errors.join('\n'));
    const report = {files: 10003, performanceFilesPerTab: 50, gridMounted, cycles: 100, tabs: 15,
      switchMedianMs: timings.sort((a,b)=>a-b)[50], switchP95Ms: timings[95],
      memoryBeforeMB: memoryBefore/1048576, memoryAfterMB: memoryAfter/1048576,
      browserPssBeforeMB: browserMemoryBefore.total/1048576, browserPssAfterMB: browserMemoryAfter.total/1048576,
      rendererPssBeforeMB: browserMemoryBefore.renderer/1048576, rendererPssAfterMB: browserMemoryAfter.renderer/1048576,
      assertions: 'file icons, selection, range, keyboard, 10k virtualization, multi-sort, resize, tab history/selection/scroll, cross-tab file move, close/reopen, missing location'};
    fs.writeFileSync(path.join(output, 'results.json'), JSON.stringify(report, null, 2));
    console.log(JSON.stringify(report, null, 2));
  } catch (error) {
    await page.screenshot({path: path.join(output, "failure.png")});
    console.log("Failure state", page.url(), (await page.locator("body").innerText()).slice(-2400));
    console.log("Storage", await page.evaluate(() => {
      try { return localStorage.getItem("wing-tabs-state"); }
      catch { return 'Storage unavailable'; }
    }));
    throw error;
  } finally {
    await browser.close();
    fs.rmSync(fixture, {recursive: true, force: true});
  }
})().catch(error => {console.error(error); process.exitCode = 1;});
