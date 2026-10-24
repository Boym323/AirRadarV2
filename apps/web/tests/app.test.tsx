import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import App from "../src/App";

describe("G-05 map-first český radar", () => {
  it("výchozí mapa má čistou kompaktní navigaci bez technických štítků a bez falešného LIVE", () => {
    const html = renderToStaticMarkup(<App />);
    expect(html).toContain(">AirRadar</h1>");
    expect(html).toContain("V2");
    expect(html).toContain("Připojování k radaru");
    expect(html).toContain("Mapa živého provozu");
    expect(html).toContain("aria-label=\"Letadla\"");
    expect(html).toContain("aria-label=\"Vrstvy\"");
    expect(html).not.toContain("Lokální ADS-B · jen potvrzené polohy");
    expect(html).not.toContain("radar-footer");
    expect(html).not.toContain("Mapový podklad");
    expect(html).not.toContain("Seznam letadel");
    expect(html).not.toContain("Vybrané letadlo");
    expect(html).not.toContain("LIVE DATA");
  });
});
