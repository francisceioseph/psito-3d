//! The atoms and molecules, driven through egui_kittest the way a person
//! (or a screen reader) would: found by their names, clicked, typed at.

use egui::Key;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use psito3d_scene::Sun;
use psito3d_ui::atoms::{Button, Slider, Toggle};
use psito3d_ui::molecules::{SeedField, SunDial};

#[test]
fn a_button_reports_clicks() {
    let mut harness = Harness::new_ui_state(
        |ui, clicks: &mut u32| {
            if ui.add(Button::new("Press me")).clicked() {
                *clicks += 1;
            }
        },
        0,
    );
    harness.get_by_label("Press me").click();
    harness.run();
    assert_eq!(*harness.state(), 1);
}

#[test]
fn a_disabled_button_ignores_clicks() {
    let mut harness = Harness::new_ui_state(
        |ui, clicks: &mut u32| {
            if ui.add(Button::new("Nope").enabled(false)).clicked() {
                *clicks += 1;
            }
        },
        0,
    );
    harness.get_by_label("Nope").click();
    harness.run();
    assert_eq!(*harness.state(), 0);
}

#[test]
fn a_slider_moves_with_the_keyboard_and_stays_in_range() {
    let mut harness = Harness::new_ui_state(
        |ui, v: &mut f32| {
            ui.add(Slider::new(v, 0.0..=1.0, "Haze"));
        },
        0.98,
    );
    harness.get_by_label("Haze").focus();
    harness.run();
    for _ in 0..5 {
        harness.key_press(Key::ArrowRight);
        harness.run();
    }
    assert_eq!(*harness.state(), 1.0, "clamped at the top");
    harness.key_press_modifiers(egui::Modifiers::SHIFT, Key::ArrowLeft);
    harness.run();
    assert!((*harness.state() - 0.9).abs() < 1e-4, "{}", harness.state());
}

#[test]
fn a_toggle_flips() {
    let mut harness = Harness::new_ui_state(
        |ui, on: &mut bool| {
            ui.add(Toggle::new(on, "Sea"));
        },
        false,
    );
    harness.get_by_label("Sea").click();
    harness.run();
    assert!(*harness.state());
}

#[test]
fn the_sun_dial_turns_and_raises_the_sun() {
    let start = Sun { azimuth: 100.0, elevation: 20.0 };
    let mut harness = Harness::new_ui_state(
        |ui, sun: &mut Sun| {
            ui.add(SunDial::new(sun, "Sun"));
        },
        start,
    );
    harness.get_by_label("Sun").focus();
    harness.run();
    harness.key_press(Key::ArrowRight);
    harness.key_press(Key::ArrowUp);
    harness.run();
    let sun = *harness.state();
    assert!(sun.azimuth > start.azimuth && sun.elevation > start.elevation, "{sun:?}");
}

#[test]
fn the_seed_field_rolls() {
    let mut harness = Harness::new_ui_state(
        |ui, (seed, rolled): &mut (u64, bool)| {
            if SeedField::new(seed, "Seed", "Roll").show(ui).rolled {
                *rolled = true;
            }
        },
        (7, false),
    );
    harness.get_by_label("Roll").click();
    harness.run();
    assert!(harness.state().1);
}
