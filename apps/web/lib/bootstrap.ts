/** Explicitly non-live G-00 state; do not display fabricated receiver data. */
export const bootstrap = Object.freeze({
  stage: "G-00",
  liveTraffic: false,
  historyAvailable: false,
  productionEnabled: false,
});
