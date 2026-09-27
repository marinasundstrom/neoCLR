package main
import ("fmt"; "strings"; "unicode/utf8")
func main() {
    text := "Ae\u0301👨‍👩‍👧‍👦🇸🇪"
    fmt.Printf("scalars=%d utf8=%d\n", utf8.RuneCountInString(text), len(text))
    fmt.Printf("canonical_equal=%t\n", "é" == "e\u0301")
    fmt.Printf("prefix_inside_grapheme=%t\n", strings.HasPrefix("e\u0301", "e"))
    fmt.Printf("simple_fold_kelvin=%t simple_fold_expansion=%t\n", strings.EqualFold("K", "k"), strings.EqualFold("ß", "ss"))
}
