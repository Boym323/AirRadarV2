import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import App from "../src/App";

describe("G-05 český radar ve Vite", () => {
  it("nepředstírá živý provoz bez SSE a poskytuje seznam i mapový kontejner", () => {
    const html = renderToStaticMarkup(<App />);
    expect(html).toContain("Není připojen živý provoz");
    expect(html).toContain("Připojování k radaru");
    expect(html).toContain("Mapa živého provozu");
    expect(html).toContain("Seznam letadel");
    expect(html).toContain("Mapový podklad");
    expect(html).toContain("AirRadar V1");
    expect(html).not.toContain("LIVE DATA");
  });
});
