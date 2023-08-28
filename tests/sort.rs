use diary_cli::sort::younger;

#[test]
fn sort_is_younger() {
    let date1 = [15, 8, 2023];
    let date2 = [30, 8, 2023];
    assert!(younger(&date2, &date1))
}