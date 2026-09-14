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
    await expect(page.locator("h1")).toContainText("Waikato Business");
    await expect(page.locator(".masthead")).toHaveCSS("position", "fixed");

    const pageHeight = await page.evaluate(() => document.documentElement.scrollHeight);
    for (let y = 0; y < pageHeight; y += Math.floor(viewport.height * 0.72)) {
      await page.evaluate((position) => window.scrollTo(0, position), y);
      await page.waitForTimeout(45);
    }
    for (const image of await page.locator("img").all()) {
      await image.scrollIntoViewIfNeeded();
      await expect(image).toHaveJSProperty("complete", true);
    }
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
