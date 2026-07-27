export function highlightJson(jsonString: string): string {
  let escapedJson = jsonString
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');

  return escapedJson.replace(
    /(&quot;(\\u[a-zA-Z0-9]{4}|\\[^u]|[^&]|&(?!quot;))*&quot;(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g,
    (match) => {
      let cls = 'hl-number';
      if (/^&quot;/.test(match)) {
        if (/:$/.test(match)) {
          cls = 'hl-key';
          // Wrap only the key name, keep the colon outside
          const keyName = match.slice(0, -1);
          return `<span class="${cls}">${keyName}</span>:`;
        } else {
          cls = 'hl-string';
        }
      } else if (/true|false/.test(match)) {
        cls = 'hl-boolean';
      } else if (/null/.test(match)) {
        cls = 'hl-null';
      }
      
      return `<span class="${cls}">${match}</span>`;
    }
  );
}

export function highlightXml(xmlString: string): string {
  // Simple regex for XML tags, attributes, and comments
  let html = xmlString
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
    
  html = html.replace(/(&lt;\/?)([a-zA-Z0-9_:\.-]+)/g, '$1<span class="hl-tag">$2</span>');
  html = html.replace(/([a-zA-Z0-9_:\.-]+)=(&quot;.*?&quot;|'.*?')/g, '<span class="hl-attr">$1</span>=<span class="hl-string">$2</span>');
  
  return html;
}

export function highlightSyntax(code: string, format: string): string {
  if (!code) return "";
  
  if (format === "json") {
    return highlightJson(code);
  } else if (format === "xml") {
    return highlightXml(code);
  }
  
  // Default: escape HTML to prevent XSS
  return code
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}
