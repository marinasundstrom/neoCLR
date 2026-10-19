use unicode_segmentation::UnicodeSegmentation;

#[test]
fn unicode_17_grapheme_conformance_in_both_directions() {
    assert_eq!(unicode_segmentation::UNICODE_VERSION, (17, 0, 0));
    let mut rows = 0;
    for line in include_str!("fixtures/grapheme-break-17.txt").lines() {
        let data = line.split('#').next().unwrap().trim();
        if data.is_empty() {
            continue;
        }
        let mut text = String::new();
        let mut boundaries = Vec::new();
        for token in data.split_whitespace() {
            match token {
                "÷" => boundaries.push(text.len()),
                "×" => (),
                scalar => {
                    text.push(char::from_u32(u32::from_str_radix(scalar, 16).unwrap()).unwrap())
                }
            }
        }
        let expected: Vec<_> = boundaries
            .windows(2)
            .map(|pair| &text[pair[0]..pair[1]])
            .collect();
        assert_eq!(
            text.graphemes(true).collect::<Vec<_>>(),
            expected,
            "row {rows}"
        );
        assert_eq!(
            text.graphemes(true).rev().collect::<Vec<_>>(),
            expected.iter().rev().copied().collect::<Vec<_>>(),
            "reverse row {rows}"
        );
        rows += 1;
    }
    assert_eq!(rows, 766);
}

#[test]
fn newer_combining_marks_remain_in_their_grapheme() {
    // Normalization conformance row 18051: U+1ACF was not an Extend
    // character in the previously pinned Unicode 16 segmentation tables.
    let left = "a\u{315}\u{300}\u{5ae}\u{1acf}b";
    let right = "a\u{5ae}\u{300}\u{1acf}\u{315}b";
    assert_eq!(left.graphemes(true).count(), 2);
    assert_eq!(right.graphemes(true).count(), 2);
    assert_eq!("a\u{1acf}".graphemes(true).count(), 1);
}
