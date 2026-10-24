/** G-05.11/R01/R03: adaptivní GeoJSON diff podložený CI benchmarkem #40095503907.
 * Při 500–2000 cílech a 10% změnách byl worker updateData rychlejší než setData.
 * Malá sada, hromadná výměna, nová epocha a změna stylu nadále používají setData.
 */
import type { Feature, FeatureCollection, Point } from "geojson";
import type { GeoJSONSource } from "maplibre-gl";

export type AircraftPatch = {
  diff: Parameters<GeoJSONSource["updateData"]>[0];
  changes: number;
};

/** Při chybějící/střetlé identitě se nesmí dělat inkrementální update. */
export function diffAircraftGeoJSON(
  before: FeatureCollection<Point>,
  after: FeatureCollection<Point>,
): AircraftPatch | null {
  const old = new Map<string | number, Feature<Point>>();
  for (const feature of before.features) {
    if (feature.id === undefined || old.has(feature.id)) return null;
    old.set(feature.id, feature);
  }
  const now = new Set<string | number>();
  const add: Feature<Point>[] = [];
  const update: NonNullable<AircraftPatch["diff"]["update"]> = [];
  for (const feature of after.features) {
    if (feature.id === undefined || now.has(feature.id)) return null;
    now.add(feature.id);
    const previous = old.get(feature.id);
    if (!previous) {
      add.push(feature);
      continue;
    }
    const moved = previous.geometry.coordinates[0] !== feature.geometry.coordinates[0] ||
      previous.geometry.coordinates[1] !== feature.geometry.coordinates[1];
    const changedProps = JSON.stringify(previous.properties) !== JSON.stringify(feature.properties);
    if (!moved && !changedProps) continue;
    update.push({
      id: feature.id,
      ...(moved ? { newGeometry: feature.geometry } : {}),
      ...(changedProps ? {
        removeAllProperties: true,
        addOrUpdateProperties: Object.entries(feature.properties ?? {})
          .map(([key, value]) => ({ key, value })),
      } : {}),
    });
  }
  const remove = [...old.keys()].filter(id => !now.has(id));
  return {
    diff: { ...(add.length ? { add } : {}), ...(update.length ? { update } : {}),
      ...(remove.length ? { remove } : {}) },
    changes: add.length + update.length + remove.length,
  };
}
export function shouldUseAircraftDiff(
  beforeCount: number, afterCount: number, changes: number, sameEpoch: boolean,
): boolean {
  return sameEpoch && beforeCount >= 250 && afterCount >= 250 &&
    changes > 0 && changes <= Math.max(1, Math.floor(afterCount * .35));
}
