use std::collections::HashMap;

pub fn fnv1a_64(input: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    let prime: u64 = 0x100000001b3; // FNV prime
    for b in input.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(prime);
    }
    hash
}
pub fn hex_u64(v: u64) -> String {
    format!("{:016x}", v)
}

#[derive(Debug)]
pub struct GtdBox {
    pub cmd: String,
    pub attrs: HashMap<String, String>,
    pub remainder: String,
}

// Parse a GTD box of the form: [@CMD:attr1=value:attr2=value] Optional text
pub fn parse_gtd_box(line: &str) -> Option<GtdBox> {
    let s = line.trim();
    if !s.starts_with('[') {
        return None;
    }
    // find matching closing bracket on the line
    let close = s.find(']')?;
    let inside = &s[1..close];
    if !inside.trim_start().starts_with('@') {
        return None;
    }
    // tokens split by ':' allowing extra spaces
    let mut tokens: Vec<String> = inside.split(':').map(|t| t.trim().to_string()).collect();
    if tokens.is_empty() {
        return None;
    }
    let first = tokens.remove(0);
    // first token begins with @
    let cmd = first.trim_start_matches('@').trim().to_string();
    let mut attrs: HashMap<String, String> = HashMap::new();
    for t in tokens {
        if t.is_empty() {
            continue;
        }
        // allow attr or attr=value; if no value, treat as flag with "true"
        let mut it = t.splitn(2, '=');
        let k = it.next().unwrap_or("").trim().to_lowercase();
        if k.is_empty() {
            continue;
        }
        let v = it.next().unwrap_or("true").trim().to_string();
        attrs.insert(k, v);
    }
    let remainder = s[close + 1..].trim().to_string();
    Some(GtdBox {
        cmd: cmd.to_string(),
        attrs,
        remainder,
    })
}

pub fn map_rank_to_priority_score(rank: &str) -> Option<i64> {
    // Accept 1-100 numeric, map linearly to 1-10; accept low|medium|high|urgent
    let lower = rank.to_lowercase();
    if let Ok(n) = lower.parse::<i64>() {
        if (1..=100).contains(&n) {
            let mut score = (n + 9) / 10; // ceil to 1..10
            score = score.clamp(1, 10);
            return Some(score);
        }
    }
    match lower.as_str() {
        "low" => Some(3),
        "medium" => Some(5),
        "high" => Some(8),
        "urgent" => Some(10),
        _ => None,
    }
}
