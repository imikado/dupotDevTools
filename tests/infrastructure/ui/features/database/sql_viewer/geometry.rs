use dupot_dev_tools::infrastructure::ui::features::database::sql_viewer::geometry::{
    bezier_point, facing_edge_x, hit_test, link_curve,
};

#[test]
fn link_curve_bends_at_the_horizontal_midpoint() {
    assert_eq!(
        link_curve((0.0, 10.0), (100.0, 50.0)),
        [(0.0, 10.0), (50.0, 10.0), (50.0, 50.0), (100.0, 50.0)]
    );
}

#[test]
fn bezier_endpoints_and_middle() {
    let curve = link_curve((0.0, 0.0), (100.0, 100.0));
    assert_eq!(bezier_point(&curve, 0.0), (0.0, 0.0));
    assert_eq!(bezier_point(&curve, 1.0), (100.0, 100.0));
    assert_eq!(bezier_point(&curve, 0.5), (50.0, 50.0));
}

#[test]
fn edge_faces_the_other_card() {
    // Card at x=0..200, the other card to its right: attach on the right edge.
    assert_eq!(facing_edge_x((0.0, 200.0), (500.0, 200.0)), 200.0);
    // The other card to its left: attach on the left edge.
    assert_eq!(facing_edge_x((500.0, 200.0), (0.0, 200.0)), 500.0);
}

#[test]
fn hit_test_picks_the_closest_curve_within_tolerance() {
    let curves = [
        link_curve((0.0, 0.0), (100.0, 0.0)),
        link_curve((0.0, 20.0), (100.0, 20.0)),
    ];
    assert_eq!(hit_test(&curves, 50.0, 3.0, 7.0), Some(0));
    assert_eq!(hit_test(&curves, 50.0, 16.0, 7.0), Some(1));
    assert_eq!(hit_test(&curves, 50.0, 60.0, 7.0), None);
    assert_eq!(hit_test(&[], 0.0, 0.0, 7.0), None);
}
