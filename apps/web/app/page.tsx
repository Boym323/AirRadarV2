import { bootstrap } from "../lib/bootstrap";

export default function HomePage() {
  return (
    <main className="shell">
      <p className="eyebrow">NOVÝ PROJEKT VE VÝVOJI · {bootstrap.stage}</p>
      <h1>AirRadar V2</h1>
      <p className="intro">
        Budujeme novou generaci leteckého radaru se samostatným Rust enginem
        a přehledným mapovým rozhraním.
      </p>
      <section className="status" aria-label="Stav vývoje">
        <h2>Není připojen živý provoz</h2>
        <p>
          Příjem letadel, mapa, sledování letů a databáze zatím nejsou
          dokončené. Jde o vývojový základ projektu, nikoli o živý radar.
        </p>
        <dl>
          <div><dt>Živá letadla</dt><dd>Zatím neimplementováno</dd></div>
          <div><dt>Historie letů</dt><dd>Databáze zatím není připravena</dd></div>
          <div><dt>Produkční nasazení</dt><dd>Vypnuto</dd></div>
        </dl>
      </section>
      <footer>AirRadar V1 běží samostatně a jeho provoz není vývojem V2 dotčen.</footer>
    </main>
  );
}
