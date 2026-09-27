fn main() {
    let text = "Ae\u{301}👨‍👩‍👧‍👦🇸🇪";
    println!("scalars={} utf8={} utf16={}", text.chars().count(), text.len(), text.encode_utf16().count());
    println!("canonical_equal={}", "é" == "e\u{301}");
    println!("prefix_inside_grapheme={}", "e\u{301}".starts_with("e"));
    println!("interior_byte_range_rejected={}", "é".get(1..2).is_none());
    println!("ascii_ignore_case_kelvin={}", "K".eq_ignore_ascii_case("k"));
}
