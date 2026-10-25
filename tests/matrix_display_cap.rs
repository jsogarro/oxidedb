//! A list of lists is shown with its columns padded to the widest cell, so one wide cell makes
//! every row as wide: the text grows as rows x width while the value stays far under
//! `MAX_LOGICAL_ITEMS`. Display must stay linear in the value.

use oxidedb::Interpreter;

#[test]
fn wide_cell_does_not_blow_up_matrix_display() {
    let mut it = Interpreter::new();
    for line in ["L:2000#(1;`a)", "m:(enlist(L;1)),10000#enlist 1 2"] {
        it.eval_line(line).unwrap();
    }
    let shown = it.eval_line("m").unwrap().unwrap().to_string();
    // the unpadded text is about 6 KB for the wide row plus 10,000 short rows
    assert!(shown.len() < 1 << 20, "display wrote {} bytes", shown.len());
}

#[test]
fn capped_matrix_shows_one_line_per_item_and_small_ones_still_pad() {
    let mut it = Interpreter::new();
    it.eval_line("L:2000#(1;`a)").unwrap();
    it.eval_line("m:(enlist(L;1)),10000#enlist 1 2").unwrap();
    let shown = it.eval_line("m").unwrap().unwrap().to_string();
    let lines: Vec<&str> = shown.lines().collect();
    assert_eq!(lines.len(), 10_001);
    assert_eq!(lines[1], "1 2");
    // under the cap the columns are still padded
    assert_eq!(
        it.eval_line("(10 2;3 4)").unwrap().unwrap().to_string(),
        "10 2\n3  4"
    );
}
