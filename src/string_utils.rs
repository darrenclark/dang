pub fn byte_to_char_index(s: &str, byte_index: usize) -> Option<usize> {
    s.char_indices().enumerate().find_map(
        |(i, (bi, _))| {
            if bi == byte_index {
                Some(i)
            } else {
                None
            }
        },
    )
}
