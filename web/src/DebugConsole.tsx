// React import not needed for JSX in modern React

type E = { t:string; level:'status'|'rotation'|'peer'|'info'|'error' };

export function DebugConsole({events}:{events:E[]}) {
  return (
    <div style={{position:'fixed',bottom:0,left:0,right:0,height:200,overflow:'auto',background:'#111',color:'#ddd',fontFamily:'ui-monospace, SFMono-Regular, Menlo, monospace',fontSize:10,borderTop:'1px solid #333',padding:'8px',lineHeight:'1.4'}}>
      {events.slice(-200).map((e,i)=>(
        <div key={i} style={{color: e.level==='error'?'#f55':e.level==='rotation'?'#f90':e.level==='peer'?'#59f':'#8f8'}}>
          [{e.level}] {e.t}
        </div>
      ))}
    </div>
  );
}
