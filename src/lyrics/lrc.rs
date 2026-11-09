//! Letras sincronizadas: lee texto LRC ("[mm:ss.xx] línea") y marca pausas largas.
//! Port de ByTune src/lib/lrc.js.

pub const GAP_TEXT: &str = "• • •";
/// El LRC no trae fin de línea: se asume que dura 4 s.
const LINE_DURATION: f64 = 4.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub t: f64,
    pub text: String,
    pub gap: bool,
}

/// Lee una marca `[mm:ss]`, `[mm:ss.x]`, `[mm:ss.xx]`, `[mm:ss.xxx]` (o con `:` en vez de `.`)
/// al principio de `s`. Devuelve los segundos y el resto del texto.
fn stamp(s: &str) -> Option<(f64, &str)> {
    let rest = s.strip_prefix('[')?;
    let close = rest.find(']')?;
    let inner = &rest[..close];
    let (min, sec_part) = inner.split_once(':')?;
    if min.is_empty() || !min.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let (sec, frac) = match sec_part.find(['.', ':']) {
        Some(i) => (&sec_part[..i], Some(&sec_part[i + 1..])),
        None => (sec_part, None),
    };
    if !(1..=2).contains(&sec.len()) || !sec.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let frac_val = match frac {
        Some(f) if (1..=3).contains(&f.len()) && f.bytes().all(|b| b.is_ascii_digit()) => {
            f.parse::<f64>().ok()? / 10f64.powi(f.len() as i32)
        }
        Some(_) => return None,
        None => 0.0,
    };
    let t = min.parse::<f64>().ok()? * 60.0 + sec.parse::<f64>().ok()? + frac_val;
    Some(((t * 1000.0).round() / 1000.0, &rest[close + 1..]))
}

pub fn parse_lrc(text: &str) -> Vec<Line> {
    let mut out = Vec::new();
    for line in text.trim_start_matches('\u{feff}').lines() {
        // Las marcas pueden ir en cualquier parte de la línea (como en el regex global de JS).
        let mut times = Vec::new();
        let mut lyric = String::new();
        let mut rest = line;
        while !rest.is_empty() {
            if let Some((t, after)) = stamp(rest) {
                times.push(t);
                rest = after;
            } else {
                let ch = rest.chars().next().unwrap();
                lyric.push(ch);
                rest = &rest[ch.len_utf8()..];
            }
        }
        if times.is_empty() {
            continue; // metadatos ([ar:...]) o texto suelto
        }
        let lyric = lyric.trim().to_string();
        for t in times {
            out.push(Line { t, text: lyric.clone(), gap: false });
        }
    }
    out.sort_by(|a, b| a.t.total_cmp(&b.t));
    out
}

pub fn add_gaps(lines: Vec<Line>, min_gap: f64) -> Vec<Line> {
    // Las líneas vacías ya son una pausa: se convierten en hueco en su sitio.
    let src: Vec<Line> = lines
        .into_iter()
        .map(|l| if l.text.is_empty() { Line { t: l.t, text: GAP_TEXT.into(), gap: true } } else { l })
        .collect();
    let mut out = Vec::with_capacity(src.len() + 4);
    if let Some(first) = src.first() {
        if !first.gap && first.t >= min_gap {
            out.push(Line { t: 0.0, text: GAP_TEXT.into(), gap: true });
        }
    }
    for (i, line) in src.iter().enumerate() {
        out.push(line.clone());
        if let Some(next) = src.get(i + 1) {
            if !line.gap && !next.gap && next.t - line.t >= min_gap + LINE_DURATION {
                out.push(Line { t: line.t + LINE_DURATION, text: GAP_TEXT.into(), gap: true });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stamps_and_skips_metadata() {
        let l = parse_lrc("\u{feff}[ar:Someone]\n[00:12.34] Hola\n[01:02] Adiós\nsuelto");
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].t, 12.34);
        assert_eq!(l[0].text, "Hola");
        assert_eq!(l[1].t, 62.0);
    }

    #[test]
    fn repeated_stamps_and_sorting() {
        let l = parse_lrc("[00:30.0][00:10.5] Coro\n[00:20.123] Verso");
        let ts: Vec<f64> = l.iter().map(|x| x.t).collect();
        assert_eq!(ts, vec![10.5, 20.123, 30.0]);
        assert!(l.iter().filter(|x| x.text == "Coro").count() == 2);
    }

    #[test]
    fn colon_fraction() {
        assert_eq!(parse_lrc("[00:01:50] x")[0].t, 1.5);
    }

    #[test]
    fn gaps() {
        let lines = parse_lrc("[00:08.00] a\n[00:09.00]\n[00:20.00] b\n[00:40.00] c");
        let g = add_gaps(lines, 6.0);
        let texts: Vec<&str> = g.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(texts, vec![GAP_TEXT, "a", GAP_TEXT, "b", GAP_TEXT, "c"]);
        assert_eq!(g[4].t, 24.0);
    }
}
