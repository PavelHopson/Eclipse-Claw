/// JavaScript snippets executed via CDP to extract design tokens.
/// Each function returns a JSON string that is parsed on the Rust side.

/// Extract all design tokens in one pass.
/// Returns a JSON object matching the DesignTokens structure (partial — Rust fills url/title).
pub const EXTRACT_TOKENS: &str = r#"
(function() {
  const all = [...document.querySelectorAll('*')];
  const sample = all.slice(0, 500);

  // --- Colors ---
  const bgCounts = new Map();
  const fgCounts = new Map();
  const borderCounts = new Map();
  const accentCounts = new Map();

  sample.forEach(el => {
    const cs = getComputedStyle(el);
    const bg = cs.backgroundColor;
    const fg = cs.color;
    const bd = cs.borderColor;

    if (bg && bg !== 'rgba(0, 0, 0, 0)' && bg !== 'transparent')
      bgCounts.set(bg, (bgCounts.get(bg) || 0) + 1);
    if (fg && fg !== 'rgba(0, 0, 0, 0)')
      fgCounts.set(fg, (fgCounts.get(fg) || 0) + 1);
    if (bd && bd !== 'rgba(0, 0, 0, 0)' && cs.borderStyle !== 'none')
      borderCounts.set(bd, (borderCounts.get(bd) || 0) + 1);

    const tag = el.tagName.toLowerCase();
    if (tag === 'button' || tag === 'a' || el.className?.toString().match(/btn|accent|primary|cta/i)) {
      accentCounts.set(bg, (accentCounts.get(bg) || 0) + 1);
    }
  });

  function topN(map, role, n) {
    return [...map.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, n)
      .map(([value, count]) => ({ value, hex: rgbToHex(value), count, role }));
  }

  function rgbToHex(rgb) {
    const m = rgb.match(/^rgba?\((\d+),\s*(\d+),\s*(\d+)/);
    if (!m) return null;
    return '#' + [m[1], m[2], m[3]].map(x => parseInt(x).toString(16).padStart(2, '0')).join('');
  }

  // --- Typography ---
  const familyCounts = new Map();
  const sizes = new Set();
  const weights = new Set();
  const lineHeights = new Set();
  const letterSpacings = new Set();

  const headings = [...document.querySelectorAll('h1,h2,h3,h4,h5,h6')];
  const bodies = [...document.querySelectorAll('p,li,span,div')].slice(0, 100);
  const codes = [...document.querySelectorAll('code,pre')];
  const labels = [...document.querySelectorAll('label,small,caption')];

  function recordFont(els, role) {
    els.forEach(el => {
      const cs = getComputedStyle(el);
      const fam = cs.fontFamily;
      if (!familyCounts.has(fam)) familyCounts.set(fam, { count: 0, roles: new Set() });
      familyCounts.get(fam).count++;
      familyCounts.get(fam).roles.add(role);
      const sz = cs.fontSize; if (sz && sz !== '0px') sizes.add(sz);
      const fw = cs.fontWeight; if (fw) weights.add(fw);
      const lh = cs.lineHeight; if (lh && lh !== 'normal') lineHeights.add(lh);
      const ls = cs.letterSpacing; if (ls && ls !== '0px' && ls !== 'normal') letterSpacings.add(ls);
    });
  }

  recordFont(headings, 'heading');
  recordFont(bodies, 'body');
  recordFont(codes, 'code');
  recordFont(labels, 'label');

  const families = [...familyCounts.entries()].sort((a, b) => b[1].count - a[1].count).slice(0, 8).map(([family, d]) => ({
    family,
    roles: [...d.roles],
    count: d.count
  }));

  // Sort sizes numerically
  const sortedSizes = [...sizes].sort((a, b) => parseFloat(a) - parseFloat(b));

  // --- Spacing ---
  const paddings = new Set();
  const gaps = new Set();
  const maxWidths = new Set();
  const sectionPads = new Set();

  const sections = [...document.querySelectorAll('section,article,[class*="section"],[class*="container"],[class*="wrapper"]')].slice(0, 50);
  sections.forEach(el => {
    const cs = getComputedStyle(el);
    const pt = cs.paddingTop; const pb = cs.paddingBottom;
    if (pt && pt !== '0px') sectionPads.add(pt);
    if (pb && pb !== '0px') sectionPads.add(pb);
    const mw = cs.maxWidth; if (mw && mw !== 'none') maxWidths.add(mw);
  });

  sample.forEach(el => {
    const cs = getComputedStyle(el);
    ['padding','paddingTop','paddingRight','paddingBottom','paddingLeft'].forEach(p => {
      const v = cs[p]; if (v && v !== '0px') paddings.add(v);
    });
    const gap = cs.gap; if (gap && gap !== '0px' && gap !== 'normal') gaps.add(gap);
  });

  // --- Border radii ---
  const radii = new Set();
  sample.forEach(el => {
    const r = getComputedStyle(el).borderRadius;
    if (r && r !== '0px') radii.add(r);
  });

  // --- Shadows ---
  const shadows = new Set();
  sample.forEach(el => {
    const s = getComputedStyle(el).boxShadow;
    if (s && s !== 'none') shadows.add(s);
  });

  // --- CSS variables ---
  const cssVars = [];
  try {
    const rootStyle = getComputedStyle(document.documentElement);
    const sheet = [...document.styleSheets].find(s => { try { s.cssRules; return true; } catch { return false; } });
    if (sheet) {
      [...sheet.cssRules].forEach(rule => {
        if (rule.selectorText === ':root') {
          rule.style.cssText.split(';').forEach(decl => {
            const [name, ...rest] = decl.split(':');
            if (name && name.trim().startsWith('--')) {
              cssVars.push({ name: name.trim(), value: rest.join(':').trim() });
            }
          });
        }
      });
    }
  } catch(e) {}

  // --- Color scheme ---
  const darkBg = [...bgCounts.keys()].some(c => {
    const m = c.match(/rgb\((\d+),\s*(\d+),\s*(\d+)/);
    if (!m) return false;
    const l = (parseInt(m[1]) * 0.299 + parseInt(m[2]) * 0.587 + parseInt(m[3]) * 0.114) / 255;
    return l < 0.4;
  });
  const colorScheme = darkBg ? 'dark' : 'light';

  // --- Scroll library ---
  const hasLenis = !![...document.querySelectorAll('[class*="lenis"]')].length ||
    !!document.querySelector('.lenis');
  const hasLoco = !!document.querySelector('.locomotive-scroll');
  const scrollLibrary = hasLenis ? 'lenis' : hasLoco ? 'locomotive-scroll' : 'none';

  // --- Font sources ---
  const fontSources = [...document.querySelectorAll('link[rel*="stylesheet"][href*="font"], style')]
    .map(el => el.href || (el.textContent?.match(/@font-face/i) ? '@font-face declarations found' : null))
    .filter(Boolean).slice(0, 10);

  return JSON.stringify({
    colors: {
      backgrounds: topN(bgCounts, 'background', 10),
      foregrounds: topN(fgCounts, 'text', 10),
      borders: topN(borderCounts, 'border', 6),
      accents: topN(accentCounts, 'accent', 6),
    },
    typography: {
      families,
      sizes: sortedSizes,
      weights: [...weights].sort(),
      line_heights: [...lineHeights].sort(),
      letter_spacings: [...letterSpacings],
    },
    spacing: {
      paddings: [...paddings].sort((a, b) => parseFloat(a) - parseFloat(b)).slice(0, 20),
      gaps: [...gaps].sort((a, b) => parseFloat(a) - parseFloat(b)).slice(0, 10),
      max_widths: [...maxWidths].slice(0, 6),
      section_paddings: [...sectionPads].sort((a, b) => parseFloat(a) - parseFloat(b)).slice(0, 8),
    },
    border_radii: [...radii].sort((a, b) => parseFloat(a) - parseFloat(b)).slice(0, 10),
    shadows: [...shadows].slice(0, 8),
    css_variables: cssVars.slice(0, 50),
    color_scheme: colorScheme,
    scroll_library: scrollLibrary,
    font_sources: fontSources,
  });
})()
"#;

/// Screenshot the page at a given viewport width.
/// Returns a base64 PNG data URL.
pub fn screenshot_script(viewport_width: u32) -> String {
    format!(
        r#"
(function() {{
  window.resizeTo({viewport_width}, window.outerHeight);
  return 'viewport set to {viewport_width}px';
}})()
"#
    )
}
