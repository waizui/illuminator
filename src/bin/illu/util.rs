
/// parse resolution with format: 800x600
pub fn parse_resolution(res: &str) -> Option<(usize, usize)> {
    let s: Vec<&str> = res.split('x').collect();
    if s.len() != 2 {
        return None;
    }

    let w = s[0].parse::<usize>().ok()?;
    let h = s[1].parse::<usize>().ok()?;
    Some((w, h))
}
