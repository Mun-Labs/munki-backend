async fn calculate_value(btc_fear: i32, sol_price_change: i32, blochain_volume: u64) -> i32 {
    let blockchain_volume_score: i32 = match blochain_volume {
        // Less than $1 billion
        0..1_000_000_000 => 10,
        // $1 billion to < $2 billion
        1_000_000_000..2_000_000_000 => 20,
        // $2 billion to < $3 billion
        2_000_000_000..3_000_000_000 => 30,
        // $3 billion to < $4 billion
        3_000_000_000..4_000_000_000 => 40,
        // $4 billion to < $5 billion
        4_000_000_000..5_000_000_000 => 50,
        // $5 billion to < $6 billion
        5_000_000_000..6_000_000_000 => 60,
        // $6 billion to < $7 billion
        6_000_000_000..7_000_000_000 => 70,
        // $7 billion to < $8 billion
        7_000_000_000..8_000_000_000 => 80,
        // $8 billion to < $10 billion (combining $8 to < $9 billion and the missing $9 to < $10 billion range)
        8_000_000_000..10_000_000_000 => 90,
        // ≥ $10 billion
        _ => 100,
    };

    (blockchain_volume_score + btc_fear) / 3
}

#[cfg(test)]
mod test {
    #[test]
    fn test_calculate_value() {
        println!("hello {}", 3 / 2);
    }
}
