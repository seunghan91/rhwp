/**
 * E2E 테스트 헬퍼 — Puppeteer + Chrome CDP
 *
 * 모드 (CLI 옵션 --mode):
 *   --mode=host    : 호스트 Windows Chrome CDP에 연결 (기본)
 *   --mode=headless: WSL2 내부 headless Chrome 실행
 *
 * 예시:
 *   node e2e/kps-ai.test.mjs                  # 호스트 Chrome CDP
 *   node e2e/kps-ai.test.mjs --mode=headless  # headless Chrome
 */
import puppeteer from 'puppeteer-core';

const CHROME_PATH = '/home/edward/.cache/puppeteer/chrome/linux-146.0.7680.31/chrome-linux64/chrome';
const CHROME_CDP = process.env.CHROME_CDP || 'http://172.21.192.1:19222';
const VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

/** CLI 인수에서 --mode=host|headless 파싱 */
function parseMode() {
  const modeArg = process.argv.find(a => a.startsWith('--mode='));
  if (modeArg) return modeArg.split('=')[1];
  return 'host';
}

const MODE = parseMode();

/** Chrome 브라우저에 연결하거나 시작하고 반환 */
export async function launchBrowser() {
  if (MODE === 'headless') {
    console.log('  [browser] headless Chrome 실행');
    return await puppeteer.launch({
      headless: true,
      executablePath: CHROME_PATH,
      args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'],
    });
  }
  // 호스트 Chrome CDP에 연결
  console.log(`  [browser] 호스트 Chrome CDP 연결 (${CHROME_CDP})`);
  const browser = await puppeteer.connect({
    browserURL: CHROME_CDP,
  });
  browser._isRemote = true;
  return browser;
}

/** 테스트용 페이지 생성 + 크기 설정
 * host 모드: width/height 미지정 시 호스트 윈도우 크기 그대로 사용
 * headless 모드: 기본 1280x900
 */
export async function createPage(browser, width, height) {
  if (MODE === 'headless') {
    const page = await browser.newPage();
    await page.setViewport({ width: width || 1280, height: height || 900 });
    return page;
  }
  // host 모드: 새 탭 열기
  const page = await browser.newPage();
  // 크기가 지정된 경우에만 윈도우 리사이즈
  if (width && height) {
    const session = await page.createCDPSession();
    const { windowId } = await session.send('Browser.getWindowForTarget');
    await session.send('Browser.setWindowBounds', {
      windowId,
      bounds: { width, height, windowState: 'normal' },
    });
    await session.detach();
    await new Promise(r => setTimeout(r, 1000));
  }
  return page;
}

/** 브라우저 정리 — CDP 연결은 disconnect, headless는 close */
export async function closeBrowser(browser) {
  if (browser._isRemote) {
    browser.disconnect();
  } else {
    await browser.close();
  }
}

/** Vite dev server에서 앱을 로드하고 WASM 초기화 완료 대기 */
export async function loadApp(page) {
  await page.goto(VITE_URL, { waitUntil: 'networkidle0', timeout: 30000 });
  // WASM 초기화 + 빈 문서 로드 완료 대기 (캔버스가 렌더링될 때까지)
  await page.waitForSelector('canvas', { timeout: 15000 });
  // 추가 안정화 대기
  await page.evaluate(() => new Promise(r => setTimeout(r, 500)));
}

/** 편집 영역(캔버스) 클릭하여 포커스 */
export async function clickEditArea(page) {
  const canvas = await page.$('canvas');
  if (!canvas) throw new Error('캔버스를 찾을 수 없습니다');
  const box = await canvas.boundingBox();
  // 캔버스 중앙보다 약간 위쪽 (본문 영역) 클릭
  await page.mouse.click(box.x + box.width / 2, box.y + 100);
  await page.evaluate(() => new Promise(r => setTimeout(r, 200)));
}

/** 키보드로 텍스트 입력 (dispatchKeyEvent 사용) */
export async function typeText(page, text) {
  for (const ch of text) {
    await page.keyboard.type(ch, { delay: 30 });
  }
  // 렌더링 안정화 대기
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
}

/** 스크린샷을 파일로 저장 */
export async function screenshot(page, name) {
  const dir = 'e2e/screenshots';
  const { mkdirSync, existsSync } = await import('fs');
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
  const path = `${dir}/${name}.png`;
  await page.screenshot({ path, fullPage: false });
  console.log(`  Screenshot: ${path}`);
  return path;
}

/** WASM bridge를 통해 페이지 수 조회 */
export async function getPageCount(page) {
  return await page.evaluate(() => {
    // @ts-ignore — 전역에 노출된 wasm bridge 접근
    return window.__wasm?.pageCount ?? document.querySelectorAll('canvas').length;
  });
}

/** WASM bridge를 통해 문단 수 조회 */
export async function getParagraphCount(page, sectionIdx = 0) {
  return await page.evaluate((sec) => {
    return window.__wasm?.getParagraphCount(sec) ?? -1;
  }, sectionIdx);
}

/** 테스트 결과 출력 헬퍼 */
export function assert(condition, message) {
  if (condition) {
    console.log(`  PASS: ${message}`);
  } else {
    console.error(`  FAIL: ${message}`);
    process.exitCode = 1;
  }
}
