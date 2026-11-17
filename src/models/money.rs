use rust_decimal::Decimal;

pub fn format_decimal(amount: Decimal) -> String {
    let s = amount.round_dp(2).to_string();
    println!("{:?}", s);
    let (num, frac) = s.split_once('.').unwrap_or((s.as_str(), "00"));
    let (sign, int) = num.strip_prefix('-').map_or(("", num), |i| ("-", i));

    let mut capacity = num.len() + frac.len() + (int.len() % 3);
    if sign == "-" {
        capacity += 1;
    }

    println!("Capacity calculated for {:?} -> {:?}", amount, capacity);

    let mut out = String::with_capacity(capacity);
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }

    format!("{sign}{out}.{frac}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    #[test]
    fn groups_thousands_and_pads_cents() {
        assert_eq!(format_decimal(dec("1234.5")), "1,234.50");
        assert_eq!(format_decimal(dec("-7")), "-7.00");
        assert_eq!(format_decimal(dec("1000000")), "1,000,000.00");
        assert_eq!(format_decimal(dec("0.006")), "0.01");
    }
}
