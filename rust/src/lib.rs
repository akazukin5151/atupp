// TODO: use serde
pub fn parse_csv_line(line: &str) -> Vec<&str> {
    line.split(',')
        .map(|x| {
            let a = x.strip_prefix('"').unwrap_or(x);
            let b = a
                .strip_suffix('"')
                .unwrap_or_else(|| a.strip_suffix("\"\n").unwrap_or(a));
            b
        })
        .collect()
}
