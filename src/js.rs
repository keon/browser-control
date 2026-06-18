use anyhow::Result;

pub const WAIT_LOAD_JS:&str="new Promise(r=>document.readyState==='complete'?r(true):addEventListener('load',()=>r(true),{once:true}))";
pub const ACTION_JS: &str = r#"({el(t){return t.startsWith('@')?document.querySelector(`[data-browser-control-ref="${t.slice(1)}"]`):document.querySelector(t)},point(t){const e=this.el(t);if(!e)throw new Error(`not found: ${t}`);e.scrollIntoView({block:'center'});const r=e.getBoundingClientRect();return{x:Math.round(r.left+r.width/2),y:Math.round(r.top+r.height/2)}},click(t){const e=this.el(t);if(!e)throw new Error(`not found: ${t}`);e.scrollIntoView({block:'center'});e.click();return true},fill(t,v){const e=this.el(t);if(!e)throw new Error(`not found: ${t}`);e.scrollIntoView({block:'center'});e.focus();const P=e.tagName==='TEXTAREA'?HTMLTextAreaElement.prototype:HTMLInputElement.prototype,D=Object.getOwnPropertyDescriptor(P,'value');D&&D.set?D.set.call(e,v):e.value=v;e.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:v}));e.dispatchEvent(new Event('change',{bubbles:true}));return true},select(t,v){const e=this.el(t);if(!e)throw new Error(`not found: ${t}`);e.value=v;e.dispatchEvent(new Event('input',{bubbles:true}));e.dispatchEvent(new Event('change',{bubbles:true}));return true}})"#;
pub const SNAPSHOT_JS: &str = r#"(()=>{const vis=e=>{const r=e.getBoundingClientRect(),s=getComputedStyle(e);return r.width>0&&r.height>0&&s.visibility!=='hidden'&&s.display!=='none'};const kind=e=>e.tagName==='A'?'link':/INPUT|TEXTAREA/.test(e.tagName)?'textbox':e.tagName==='BUTTON'?'button':e.getAttribute('role')||'element';const label=e=>(e.innerText||e.value||e.placeholder||e.ariaLabel||e.title||'').trim().replace(/\s+/g,' ').slice(0,100);const q='a,button,input,textarea,select,[role],[onclick],[tabindex],summary,[contenteditable=true]';const elements=[...document.querySelectorAll(q)].filter(vis).slice(0,200).map((e,i)=>{const ref=`e${i+1}`;e.setAttribute('data-browser-control-ref',ref);const r=e.getBoundingClientRect();return{ref,kind:kind(e),text:label(e),x:Math.round(r.x),y:Math.round(r.y),w:Math.round(r.width),h:Math.round(r.height)}});return{url:location.href,title:document.title,elements}})()"#;
pub const PAGE_INFO_JS: &str = "({url:location.href,title:document.title,readyState:document.readyState,w:innerWidth,h:innerHeight,sx:scrollX,sy:scrollY,pw:document.documentElement.scrollWidth,ph:document.documentElement.scrollHeight})";

pub fn inspect_js(query: Option<&str>, limit: usize, text_limit: usize) -> Result<String> {
    let q = serde_json::to_string(&query.unwrap_or(""))?;
    Ok(format!(
        r#"(()=>{{const needle={q}.toLowerCase(),limit={limit},textLimit={text_limit};
const clean=s=>(s||'').replace(/\s+/g,' ').trim();
const vis=e=>{{const r=e.getBoundingClientRect(),s=getComputedStyle(e);return r.width>0&&r.height>0&&s.visibility!=='hidden'&&s.display!=='none'}};
const kind=e=>e.tagName==='A'?'link':/INPUT|TEXTAREA/.test(e.tagName)?'textbox':e.tagName==='SELECT'?'select':e.tagName==='BUTTON'?'button':e.getAttribute('role')||'element';
const body=clean(document.body&&document.body.innerText);
let text=body.slice(0,textLimit);
if(needle){{const lo=body.toLowerCase(),hits=[];let at=lo.indexOf(needle),last=-9999;while(at>=0&&hits.length<6){{if(at>last+120){{hits.push(body.slice(Math.max(0,at-180),Math.min(body.length,at+needle.length+320)));last=at}}at=lo.indexOf(needle,at+needle.length)}}text=hits.join('\n---\n')||text}}
const headings=[...document.querySelectorAll('h1,h2,h3')].map(e=>clean(e.innerText)).filter(Boolean).slice(0,20);
const qsel='a,button,input,textarea,select,[role],[onclick],[tabindex],summary,[contenteditable=true]';
let elements=[...document.querySelectorAll(qsel)].filter(vis).map((e,i)=>{{const ref=`e${{i+1}}`;e.setAttribute('data-browser-control-ref',ref);const r=e.getBoundingClientRect(),txt=clean(e.innerText||e.value||e.placeholder||e.ariaLabel||e.title||e.getAttribute('name')||''),href=e.href||'';return{{ref,kind:kind(e),text:txt.slice(0,140),href,x:Math.round(r.x),y:Math.round(r.y),w:Math.round(r.width),h:Math.round(r.height)}}}});
if(needle)elements=elements.filter(e=>(e.text+' '+e.href+' '+e.kind).toLowerCase().includes(needle));
return{{url:location.href,title:document.title,readyState:document.readyState,query:{q},text,headings,elements:elements.slice(0,limit)}}}})()"#
    ))
}

/// `eval 'const x = 1; return x'` works like `eval 'document.title'`: a
/// top-level `return` (outside strings/comments) gets IIFE-wrapped.
pub fn wrap_return(expression: &str) -> String {
    if has_top_level_return(expression) && !expression.trim_start().starts_with('(') {
        format!("(()=>{{{expression}}})()")
    } else {
        expression.to_string()
    }
}

fn has_top_level_return(src: &str) -> bool {
    let b: Vec<char> = src.chars().collect();
    let n = b.len();
    let is_word = |c: char| c == '_' || c.is_alphanumeric();
    let (mut i, mut quote) = (0, ' ');
    let mut state = 0u8; // 0 code, 1 line comment, 2 block comment, 3 string
    while i < n {
        let ch = b[i];
        let nxt = if i + 1 < n { b[i + 1] } else { '\0' };
        match state {
            0 => {
                if ch == '\'' || ch == '"' || ch == '`' {
                    state = 3;
                    quote = ch;
                } else if ch == '/' && nxt == '/' {
                    state = 1;
                    i += 1;
                } else if ch == '/' && nxt == '*' {
                    state = 2;
                    i += 1;
                } else if b[i..].starts_with(&['r', 'e', 't', 'u', 'r', 'n'])
                    && (i == 0 || !is_word(b[i - 1]))
                    && (i + 6 >= n || !is_word(b[i + 6]))
                {
                    return true;
                }
            }
            1 => {
                if ch == '\n' {
                    state = 0;
                }
            }
            2 => {
                if ch == '*' && nxt == '/' {
                    state = 0;
                    i += 1;
                }
            }
            _ => {
                if ch == '\\' {
                    i += 1;
                } else if ch == quote {
                    state = 0;
                }
            }
        }
        i += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_top_level_return() {
        assert_eq!(
            wrap_return("const x=1; return x"),
            "(()=>{const x=1; return x})()"
        );
        assert_eq!(wrap_return("document.title"), "document.title");
        assert_eq!(wrap_return("'return'"), "'return'");
        assert_eq!(wrap_return("// return\n1"), "// return\n1");
        assert_eq!(wrap_return("/* return */ 1"), "/* return */ 1");
        assert_eq!(wrap_return("returned"), "returned");
        let already = "(()=>{return 1})()";
        assert_eq!(wrap_return(already), already);
    }

    #[test]
    fn inspect_js_embeds_query_safely() {
        let js = inspect_js(Some(r#"x"y"#), 7, 123).unwrap();
        assert!(js.contains(r#"const needle="x\"y".toLowerCase()"#));
        assert!(js.contains("limit=7"));
        assert!(js.contains("textLimit=123"));
    }
}
