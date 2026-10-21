import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import App from "../src/App";

describe("česká vstupní stránka Vite", () => {
  it("nepředstírá živý radar ani produkční nasazení", () => {
    const html = renderToStaticMarkup(<App />);
    expect(html).toContain("Není připojen živý provoz");
    expect(html).toContain("Zatím neimplementováno");
    expect(html).toContain("Produkční nasazení");
    expect(html).not.toContain("LIVE DATA");
  });
});
