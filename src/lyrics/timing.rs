//! Tiempos de letras sincronizadas. Port de ByTune src/lib/lyrics-timing.js.

/// Índice de la última línea con inicio <= t (None antes de la primera). Búsqueda binaria.
pub fn active_index(times: &[f64], t: f64) -> Option<usize> {
    let n = times.partition_point(|&x| x <= t);
    n.checked_sub(1)
}

/// Milisegundos hasta la siguiente línea (según la velocidad); None si no hay más.
pub fn ms_until_next(times: &[f64], t: f64, rate: f64) -> Option<u64> {
    if !(rate > 0.0) || !t.is_finite() {
        return None;
    }
    let next = active_index(times, t).map_or(0, |i| i + 1);
    let target = *times.get(next)?;
    // Redondeo a 6 decimales antes del ceil para evitar ruido de coma flotante.
    let ms = (((target - t) * 1000.0 / rate) * 1e6).round() / 1e6;
    Some(ms.max(0.0).ceil() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index() {
        let t = [1.0, 2.0, 3.0];
        assert_eq!(active_index(&t, 0.5), None);
        assert_eq!(active_index(&t, 1.0), Some(0));
        assert_eq!(active_index(&t, 2.5), Some(1));
        assert_eq!(active_index(&t, 9.0), Some(2));
    }

    #[test]
    fn until_next() {
        let t = [1.0, 2.0];
        assert_eq!(ms_until_next(&t, 0.0, 1.0), Some(1000));
        assert_eq!(ms_until_next(&t, 1.9, 2.0), Some(50));
        assert_eq!(ms_until_next(&t, 2.0, 1.0), None);
        assert_eq!(ms_until_next(&t, 1.0, 0.0), None);
    }
}
