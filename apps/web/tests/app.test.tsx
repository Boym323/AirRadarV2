import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import App from "../src/App";

describe("G-05 map-first český radar", () => {
  it("mapa je výchozí plochou bez trvale otevřeného seznamu a bez falešného LIVE", () => {
    const html = renderToStaticMarkup(<App />);
    expect(html).toContain("AirRadar V2");
    expect(html).toContain("Připojování k radaru");
    expect(html).toContain("Mapa živého provozu");
    expect(html).toContain("Letadla");
    expect(html).toContain("Mapový podklad");
    expect(html).toContain("Lokální ADS-B");
    expect(html).not.toContain("Seznam letadel");
    expect(html).not.toContain("Vybrané letadlo");
    expect(html).not.toContain("LIVE DATA");
  });
});
