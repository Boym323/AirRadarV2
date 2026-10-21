import { bootstrap } from "../lib/bootstrap";

export default function HomePage() {
  return (
    <main className="shell">
      <p className="eyebrow">GREENFIELD DEVELOPMENT · {bootstrap.stage}</p>
      <h1>AirRadar V2</h1>
      <p className="intro">Building a new aircraft radar with an independent Rust engine and a clean map interface.</p>
      <section className="status" aria-label="Build status">
        <h2>Not connected to live aircraft</h2>
        <p>The engine, map, tracking and database are not yet operational. This page is an initial project scaffold, not a live radar.</p>
        <dl>
          <div><dt>Live traffic</dt><dd>Not implemented</dd></div>
          <div><dt>History database</dt><dd>Not configured</dd></div>
          <div><dt>Production deployment</dt><dd>Disabled</dd></div>
        </dl>
      </section>
      <footer>AirRadar V1 remains unaffected by this separate project.</footer>
    </main>
  );
}
