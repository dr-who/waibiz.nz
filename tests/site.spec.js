const { test, expect } = require("@playwright/test");

const viewports = [
  { name: "desktop", width: 1440, height: 900 },
  { name: "mobile", width: 390, height: 844 },
];

for (const viewport of viewports) {
  test(`${viewport.name} journey renders cleanly`, async ({ page }) => {
    const errors = [];
    page.on("console", (message) => {
      if (message.type() === "error") errors.push(message.text());
    });
    page.on("pageerror", (error) => errors.push(error.message));

    await page.setViewportSize(viewport);
    await page.goto("/", { waitUntil: "networkidle" });
    await expect(page.locator("h1")).toContainText("Entrepreneur Meetup");
    await expect(page.locator(".masthead")).toHaveCSS("position", "fixed");
    await expect(page.locator(".river-town")).toHaveCount(13);
    await expect(page.locator(".river-town--major")).toContainText("Kirikiriroa");
    await expect(page.locator(".bend__media img").first()).toHaveCSS("opacity", "0.54");
    await expect(page.locator(".event-facts")).toContainText("Thursday 15 October 2026");

    const pageHeight = await page.evaluate(() => document.documentElement.scrollHeight);
    for (let y = 0; y < pageHeight; y += Math.floor(viewport.height * 0.72)) {
      await page.evaluate((position) => window.scrollTo(0, position), y);
      await page.waitForTimeout(45);
    }
    for (const image of await page.locator("img").all()) {
      await image.scrollIntoViewIfNeeded();
      await expect(image).toHaveJSProperty("complete", true);
    }
    await page.evaluate(() => window.scrollTo(0, document.documentElement.scrollHeight));
    await page.waitForTimeout(180);
    const riverMouth = await page.locator("#river-field").evaluate((canvas) => {
      const context = canvas.getContext("2d");
      const scale = canvas.width / canvas.clientWidth;
      const mouthInset = canvas.clientWidth < 700 ? 70 : 34;
      const bandTop = Math.max(0, Math.floor(canvas.height - (mouthInset + 90) * scale));
      const bandBottom = Math.min(canvas.height, Math.ceil(canvas.height - (mouthInset - 20) * scale));
      const bandHeight = bandBottom - bandTop;
      const pixels = context.getImageData(0, bandTop, canvas.width, bandHeight).data;
      let minimumX = canvas.width;
      let maximumX = 0;
      let minimumY = bandHeight;
      let maximumY = 0;
      for (let y = 0; y < bandHeight; y += 2) {
        for (let x = 0; x < canvas.width; x += 2) {
          if (pixels[(y * canvas.width + x) * 4 + 3] > 30) {
            minimumX = Math.min(minimumX, x);
            maximumX = Math.max(maximumX, x);
            minimumY = Math.min(minimumY, y);
            maximumY = Math.max(maximumY, y);
          }
        }
      }
      return {
        centre: (minimumX + maximumX) / 2,
        height: maximumY - minimumY,
        riverWidth: maximumX - minimumX,
        width: canvas.width,
      };
    });
    await page.screenshot({ path: `/tmp/waibiz-${viewport.name}-port.png` });
    expect(riverMouth.centre).toBeLessThan(riverMouth.width * 0.4);
    expect(riverMouth.riverWidth).toBeGreaterThan(riverMouth.height * 1.1);
    await page.evaluate(() => window.scrollTo(0, 0));
    await page.waitForTimeout(180);

    const layout = await page.evaluate(() => ({
      width: innerWidth,
      scrollWidth: document.documentElement.scrollWidth,
      images: [...document.images].map((image) => ({
        src: image.currentSrc,
        complete: image.complete,
        width: image.naturalWidth,
      })),
      canvases: [...document.querySelectorAll("canvas")].map((canvas) => {
        const pixels = canvas.getContext("2d").getImageData(0, 0, canvas.width, canvas.height).data;
        let nonTransparent = 0;
        const stride = Math.max(4, Math.floor(pixels.length / 12_000 / 4) * 4);
        for (let index = 3; index < pixels.length; index += stride) {
          if (pixels[index] > 0) nonTransparent += 1;
        }
        return { id: canvas.id, width: canvas.width, height: canvas.height, nonTransparent };
      }),
      wasm: performance.getEntriesByType("resource").some((entry) => entry.name.endsWith(".wasm")),
    }));

    expect(layout.scrollWidth).toBeLessThanOrEqual(layout.width + 1);
    expect(layout.wasm).toBeTruthy();
    expect(layout.images.every((image) => image.complete && image.width > 0)).toBeTruthy();
    expect(layout.canvases).toHaveLength(2);
    expect(layout.canvases.every((canvas) => canvas.width > 0 && canvas.height > 0 && canvas.nonTransparent > 0)).toBeTruthy();

    await page.locator(".stage-picker button").nth(2).click();
    await expect(page.locator(".stage-picker__answer")).toContainText("knot");
    expect(errors).toEqual([]);

    await page.screenshot({
      path: `/tmp/waibiz-${viewport.name}-travelled.png`,
      fullPage: true,
    });
  });
}
