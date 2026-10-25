<maint:doc xmlns:maint="http://www.hickorydocs.com/1.0">
# Maintain output/coverage-debug-example.html

For: maintainers changing the literate workflow and reviewers checking migration evidence.

Edit this entry, regenerate, and commit its output with the Markdown. Historical evidence retains its original scope.

<maint:entry path="output/coverage-debug-example.html">
<div class="viz-dotted-background" style="padding:16px">
<section id="coverage-debug-demo" aria-label="Illustrative coverage and debugging interface">
<style>
#coverage-debug-demo{color-scheme:light dark;--paper:light-dark(#ffffff,#202124);--ink:light-dark(#20252a,#eceef0);--edge:light-dark(#d8dce0,#45494d);--soft:light-dark(#f4f5f6,#2b2e32);--hit:light-dark(#e3f2e9,#233d30);--miss:light-dark(#fff0de,#493820);background:var(--paper);color:var(--ink);border:1px solid var(--edge);border-radius:12px;padding:20px;font:14px/1.6 system-ui,sans-serif;max-width:700px;margin:auto}
#coverage-debug-demo h3{font:500 18px/1.4 system-ui;margin:0 0 4px}#coverage-debug-demo .meta{font-size:12px;opacity:.8}#coverage-debug-demo .toolbar{display:flex;align-items:center;gap:12px;flex-wrap:wrap;margin:18px 0 12px}#coverage-debug-demo select,#coverage-debug-demo button{font:inherit;color:var(--ink);background:var(--paper);border:1px solid var(--edge);border-radius:6px;padding:7px 10px}#coverage-debug-demo .code{font:13px/1.8 ui-monospace,monospace;border:1px solid var(--edge);border-radius:6px;overflow-wrap:anywhere}#coverage-debug-demo .line{display:grid;grid-template-columns:24px 24px minmax(0,1fr);padding:2px 8px;white-space:pre-wrap}#coverage-debug-demo .hit{background:var(--hit)}#coverage-debug-demo .miss{background:var(--miss)}#coverage-debug-demo .ln{opacity:.65}#coverage-debug-demo .evidence{margin-top:14px;padding:14px;background:var(--soft);border-radius:6px}#coverage-debug-demo .heading{font-weight:500;margin-bottom:6px}#coverage-debug-demo .vars{display:flex;gap:20px;flex-wrap:wrap;margin:8px 0}#coverage-debug-demo code{font-family:ui-monospace,monospace}#coverage-debug-demo [hidden]{display:none}#coverage-debug-demo button:focus-visible,#coverage-debug-demo select:focus-visible{outline:2px solid var(--ink);outline-offset:2px}
</style>
<h3>Reject negative quantities</h3>
<div class="meta">checkout.md · revision a1b2c3 · Illustrative data</div>
<div class="toolbar"><label>Coverage group <select aria-label="Coverage group"><option value="adversarial">Adversarial</option><option value="happy">Happy path</option></select></label><span class="summary" aria-live="polite"></span></div>
<div class="code" aria-label="Python source with coverage">
<div class="line"><span class="ln">12</span><span>·</span><span>def total(qty, price):</span></div>
<div class="line" data-line="13"><span class="ln">13</span><span class="mark"></span><span>    if qty &lt; 0:</span></div>
<div class="line" data-line="14"><span class="ln">14</span><span class="mark"></span><span>        raise ValueError("negative")</span></div>
<div class="line" data-line="15"><span class="ln">15</span><span class="mark"></span><span>    return qty * price</span></div>
</div>
<div class="meta" style="margin-top:8px">✓ Covered &nbsp; — Missed &nbsp; · Outside measured statements</div>
<div class="evidence" aria-live="polite"><div class="heading">Coverage report · recorded test run</div><div class="coverage-detail"></div></div>
<div class="toolbar"><button type="button" class="cursor-interaction" aria-expanded="false">Show DAP capture at line 14</button></div>
<div class="evidence capture" hidden><div class="heading">DAP capture · separate debug run D7</div><div class="meta">Same revision a1b2c3 · test_rejects_negative · adversarial</div><div>Paused before line 14 · frame <code>total</code></div><div class="vars"><code>qty = -2</code><code>price = 10</code><code>qty &lt; 0 → True</code></div><div class="meta">One recorded stop. Coverage counts come from the coverage report.</div></div>
</section></div>
<script>
(()=>{const root=document.getElementById('coverage-debug-demo');const select=root.querySelector('select');const button=root.querySelector('button');const capture=root.querySelector('.capture');let expanded=false;
function render(){const adversarial=select.value==='adversarial';root.querySelectorAll('[data-line]').forEach(row=>{const hit=row.dataset.line==='13'||(adversarial?row.dataset.line==='14':row.dataset.line==='15');row.className='line '+(hit?'hit':'miss');row.querySelector('.mark').textContent=hit?'✓':'—';});root.querySelector('.summary').textContent='2 / 3 measured statements';root.querySelector('.coverage-detail').textContent=adversarial?'test_rejects_negative · Passed · rejection path exercised; return missed.':'test_valid_quantity · Passed · return exercised; rejection path missed.';capture.hidden=!expanded;button.setAttribute('aria-expanded',String(expanded));button.textContent=(expanded?'Hide':'Show')+' DAP capture at line 14';}
function save(){if(window.openai?.setWidgetState)window.openai.setWidgetState({modelContent:{coverageGroup:select.value,dapCaptureVisible:expanded},privateContent:null}).catch(()=>{});}
function restore(state){const s=state?.modelContent;if(s){select.value=s.coverageGroup==='happy'?'happy':'adversarial';expanded=s.dapCaptureVisible===true;}render();}
select.addEventListener('change',()=>{render();save();});button.addEventListener('click',()=>{expanded=!expanded;render();save();});window.addEventListener('openai:set_globals',e=>{if(e.detail?.globals?.widgetState)restore(e.detail.globals.widgetState);});restore(window.openai?.widgetState);
})();
</script>
</maint:entry>
</maint:doc>
