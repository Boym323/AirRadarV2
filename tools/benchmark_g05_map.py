#!/usr/bin/env python3
"""G-05.11 — reálný Chromium/MapLibre render benchmark, headless SwiftShader.
Měří browser engine na výslovně uvedeném CI runneru; NENÍ fyzický GPU test.
"""
import json
import os
import statistics
import subprocess
import time
from contextlib import closing
from pathlib import Path
from urllib.request import urlopen
from playwright.sync_api import sync_playwright, expect
from test_g05_browser import MOCK, STYLE, URL

JS = r"""async ({count, hz, method}) => {
  const map = window.__g05BenchMap;
  if (!map || !map.isStyleLoaded()) throw new Error('MapLibre benchmark hook nenalezen');
  const source = map.getSource('radar-aircraft');
  if (!source || typeof source.updateData !== 'function') throw new Error('GeoJSONSource.updateData není k dispozici');
  const features = Array.from({length:count},(_,i)=>({
    type:'Feature',id:'icao:'+i.toString(16).padStart(6,'0'),
    geometry:{type:'Point',coordinates:[16.5+(i%55)*.025,48.2+(i%37)*.028]},
    properties:{id:'icao:'+i.toString(16).padStart(6,'0'),
      label:'OK'+i.toString().padStart(5,'0'),detailLabel:'OK'+i.toString().padStart(5,'0')+'\\n5000 ft',
      heading:(i*7)%360,icon:i%3?'generic-fixedwing':'rotorcraft',selected:false}
  }));
  await source.setData({type:'FeatureCollection',features});
  const times=[],raf=[], interval=1000/hz;
  const heapStart=performance.memory?.usedJSHeapSize??null;
  const onFrame=time=>{raf.push(time); if (raf.length < 600) requestAnimationFrame(onFrame)};
  requestAnimationFrame(onFrame);
  const observer=new MutationObserver(()=>{domMutations++});
  let domMutations=0;
  observer.observe(document.querySelector('.radar-page'),{childList:true,subtree:true});
  let last=performance.now();
  for(let tick=0;tick<4;tick++) {
    const until=last+interval;
    await new Promise(resolve=>setTimeout(resolve,Math.max(0,until-performance.now())));
    last=performance.now();
    const change=[];
    for(let i=0;i<count;i+=10) {
      const feature=features[i];
      const coord=feature.geometry.coordinates;
      const next=[coord[0]+0.0001,coord[1]+0.0001];
      feature.geometry={type:'Point',coordinates:next};
      change.push({id:feature.id,newGeometry:feature.geometry});
    }
    const t=performance.now();
    if(method==='setData') await source.setData({type:'FeatureCollection',features});
    else await source.updateData({update:change});
    times.push(performance.now()-t);
  }
  observer.disconnect();
  const gaps=raf.slice(1).map((x,i)=>x-raf[i]).filter(x=>x>0 && x<2000);
  gaps.sort((a,b)=>a-b);
  times.sort((a,b)=>a-b);
  const pct=(values,p)=>values.length?values[Math.min(values.length-1,Math.ceil(values.length*p)-1)]:null;
  const heapEnd=performance.memory?.usedJSHeapSize??null;
  return {
    count,hz,method,ticks:times.length,p95UpdateMs:pct(times,.95),
    medianUpdateMs:pct(times,.50),p95FrameMs:pct(gaps,.95),
    estimatedFps:gaps.length?Math.round(1000/(gaps.reduce((a,b)=>a+b,0)/gaps.length)):null,
    heapStartBytes:heapStart,heapEndBytes:heapEnd,
    domMutations,domMarkers:document.querySelectorAll('.maplibregl-marker').length,
    mapInstances:document.querySelectorAll('.maplibregl-canvas').length
  };
}"""

def main():
    server = subprocess.Popen(["npm","run","preview","--prefix","apps/web"],
                              stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    try:
        for _ in range(80):
            try:
                with closing(urlopen(URL,timeout=1)) as response:
                    if response.status==200: break
            except OSError: time.sleep(.25)
        else: raise RuntimeError("Vite preview nenastartovalo")
        with sync_playwright() as p:
            browser=p.chromium.launch(channel="chrome",headless=True,
                args=["--enable-webgl","--use-gl=angle","--use-angle=swiftshader",
                      "--enable-unsafe-swiftshader","--enable-precise-memory-info"])
            results=[]
            for label,viewport,mobile in [
                ("desktop-1440x900",{"width":1440,"height":900},False),
                ("mobile-emulation-390x844",{"width":390,"height":844},True)
            ]:
                page=browser.new_page(viewport=viewport,is_mobile=mobile,has_touch=mobile)
                page.add_init_script(MOCK)
                page.route("https://tiles.openfreemap.org/**",
                    lambda route:route.fulfill(status=200,content_type="application/json",body=json.dumps(STYLE)))
                page.goto(URL+"?g05-benchmark=1")
                expect(page.locator(".maplibregl-canvas")).to_be_visible(timeout=12000)
                page.wait_for_function("!!window.__g05BenchMap && window.__g05BenchMap.isStyleLoaded()",
                                       timeout=12000)
                cdp=page.context.new_cdp_session(page)
                cdp.send("Performance.enable")
                for count in [25,100,500,2000]:
                    for hz in [1,2]:
                        for method in ["setData","updateData"]:
                            before={x["name"]:x["value"] for x in cdp.send("Performance.getMetrics")["metrics"]}
                            item=page.evaluate(JS,{"count":count,"hz":hz,"method":method})
                            after={x["name"]:x["value"] for x in cdp.send("Performance.getMetrics")["metrics"]}
                            item["profile"]=label
                            item["browser"]="Google Chrome headless / SwiftShader / GitHub ubuntu-24.04"
                            item["taskDurationMs"]=round(1000*(after.get("TaskDuration",0)-before.get("TaskDuration",0)),2)
                            assert item["ticks"]==4
                            assert item["mapInstances"]==1 and item["domMarkers"]==0
                            assert item["p95UpdateMs"] is not None and item["p95UpdateMs"]<5000, item
                            assert item["p95FrameMs"] is not None and item["p95FrameMs"]<2000, item
                            results.append(item)
                            print("G05_BENCH="+json.dumps(item,ensure_ascii=False),flush=True)
                page.close()
            browser.close()
        output={"schema":1,"basis":"Headless Chromium SwiftShader. Není měření na fyzickém GPU.",
                "commit":os.getenv("GITHUB_SHA","local"),"results":results}
        path=Path(".artifacts/g05-map-benchmark.json")
        path.parent.mkdir(exist_ok=True)
        path.write_text(json.dumps(output,ensure_ascii=False,indent=2),encoding="utf8")
        print("PASS G-05.11 Chromium render measurement: "+
              str(len(results))+" scénářů; fyzické GPU zůstává NEOVĚŘENO",flush=True)
    finally:
        server.terminate()
        try: server.wait(timeout=5)
        except subprocess.TimeoutExpired: server.kill();server.wait()

if __name__=="__main__":
    main()
