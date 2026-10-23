(function(){
if(/[?&]theme=dark/.test(location.search))document.documentElement.dataset.theme='dark';
const P={
 doc:'<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5"/>',
 mic:'<rect x="9" y="3" width="6" height="11" rx="3"/><path d="M5 11a7 7 0 0 0 14 0M12 18v3"/>',
 play:'<path d="M7 5l12 7-12 7z"/>', pause:'<path d="M8 5v14M16 5v14"/>', stop:'<rect x="6" y="6" width="12" height="12" rx="1.5"/>',
 speaker:'<path d="M4 9h4l5-4v14l-5-4H4z"/><path d="M16.5 9a4.5 4.5 0 0 1 0 6M19 6.5a8 8 0 0 1 0 11"/>',
 lib:'<path d="M4 6h16M4 12h16M4 18h10"/>', box:'<path d="M12 3l8 4.5v9L12 21l-8-4.5v-9z"/><path d="M12 12l8-4.5M12 12v9M12 12L4 7.5"/>',
 gear:'<circle cx="12" cy="12" r="3"/><path d="M12 3v2.5M12 18.5V21M3 12h2.5M18.5 12H21M5.6 5.6l1.8 1.8M16.6 16.6l1.8 1.8M18.4 5.6l-1.8 1.8M7.4 16.6l-1.8 1.8"/>',
 down:'<path d="M12 4v12M7 11l5 5 5-5M5 20h14"/>', check:'<path d="M5 12.5l4.5 4.5L19 7.5"/>', x:'<path d="M6 6l12 12M18 6L6 18"/>',
 search:'<circle cx="11" cy="11" r="6"/><path d="M20 20l-4.5-4.5"/>', chev:'<path d="M6 9l6 6 6-6"/>', folder:'<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
 copy:'<rect x="8" y="8" width="12" height="12" rx="2"/><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"/>',
 plus:'<path d="M12 5v14M5 12h14"/>', clock:'<circle cx="12" cy="12" r="8.5"/><path d="M12 7v5l3 2"/>', users:'<circle cx="9" cy="8" r="3.2"/><path d="M3 20a6 6 0 0 1 12 0M16 5a3.2 3.2 0 0 1 0 6M21 20a6 6 0 0 0-4-5.6"/>',
 trash:'<path d="M4 7h16M10 7V4h4v3M6 7l1 13h10l1-13"/>', dots:'<circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/>',
 globe:'<circle cx="12" cy="12" r="8.5"/><path d="M3.5 12h17M12 3.5c3 3.2 3 13.8 0 17M12 3.5c-3 3.2-3 13.8 0 17"/>',
 kbd:'<rect x="3" y="6" width="18" height="12" rx="2"/><path d="M7 10h.01M11 10h.01M15 10h.01M7 14h10"/>',
 text:'<path d="M5 6h14M12 6v13M9 19h6"/>', skip:'<path d="M18 5v14M6 5l9 7-9 7z"/>', back:'<path d="M6 5v14M18 5l-9 7 9 7z"/>',
 edit:'<path d="M4 20l1-4L16 5l3 3L8 19z"/>', disk:'<ellipse cx="12" cy="6" rx="8" ry="3"/><path d="M4 6v12c0 1.7 3.6 3 8 3s8-1.3 8-3V6M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3"/>',
 lock:'<rect x="5" y="11" width="14" height="9" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/>', refresh:'<path d="M20 11a8 8 0 0 0-14-4M4 5v4h4M4 13a8 8 0 0 0 14 4M20 19v-4h-4"/>',
 wave:'<path d="M4 12v0M8 8v8M12 4v16M16 8v8M20 12v0"/>',
 monitor:'<rect x="3" y="4" width="18" height="12" rx="2"/><path d="M9 20h6M12 16v4"/>',
 sun:'<circle cx="12" cy="12" r="4"/><path d="M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6L7 7M17 17l1.4 1.4M18.4 5.6L17 7M7 17l-1.4 1.4"/>',
 moon:'<path d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z"/>'
};
window.icon=(n,cls)=>`<svg class="i ${cls||''}" viewBox="0 0 24 24">${P[n]||''}</svg>`;
function rng(s){return()=>(s=(s*1664525+1013904223)>>>0)/4294967296}
window.waveHTML=(n,h,seed,opt)=>{opt=opt||{};const r=rng(seed||7);let o='';for(let k=0;k<n;k++){const env=0.35+0.65*Math.abs(Math.sin(k/9+r()*0.8))*(0.5+r()*0.5);const hh=Math.max(2,Math.round(h*env));const played=opt.played!=null&&k<opt.played;o+=`<i style="height:${hh}px;${played?'background:var(--accent)':''}"></i>`}return o};
document.querySelectorAll('[data-i]').forEach(e=>e.outerHTML=icon(e.dataset.i,e.className));
document.querySelectorAll('[data-wave]').forEach(e=>{const [n,h,s,p]=e.dataset.wave.split(',').map(Number);e.innerHTML=waveHTML(n,h,s,{played:isNaN(p)?null:p})});
document.querySelectorAll('[data-themeseg] span').forEach(e=>e.classList.toggle('on',e.dataset.v===(document.documentElement.dataset.theme==='dark'?'dark':'system')));
const b=document.body, a=b.dataset.active, t=b.dataset.title||'Voice Studio';
if(b.dataset.shell==='none')return;
const pref=document.documentElement.dataset.theme==='dark'?'dark':'system';
const nav=[['transcribe','doc','Transcribe',''],['speak','speaker','Speak',''],['library','lib','Library','42'],['models','box','Models','']];
const side=`<div class="sidebar"><div class="brand">Voice Studio</div><div class="nav">${nav.map(n=>`<a class="${a===n[0]?'on':''}">${icon(n[1])}${n[2]}${n[3]?`<span class="count">${n[3]}</span>`:''}</a>`).join('')}</div><div class="grow"></div><div class="themesw">${[['system','monitor','System'],['light','sun','Light'],['dark','moon','Dark']].map(t=>`<span class="${t[0]===pref?'on':''}" title="${t[2]}">${icon(t[1])}</span>`).join('')}</div><div class="nav"><a class="${a==='settings'?'on':''}">${icon('gear')}Settings</a></div></div>`;
const main=document.querySelector('main');
const status=b.dataset.status||'Whisper base · Metal GPU';
const win=document.createElement('div');win.className='win';
win.innerHTML=`<div class="titlebar"><span class="dot"></span><span class="dot"></span><span class="dot"></span><span class="ttl">${t}</span></div><div class="body">${side}<div class="main"></div></div>`;
const m=win.querySelector('.main');m.appendChild(main);
m.insertAdjacentHTML('beforeend',`<div class="status"><span><span class="ok"></span>Offline — everything runs on this device</span><span class="sp"></span><span>${status}</span></div>`);
b.prepend(win);
})();
