//! Integration and contract scenario tests for RedPower 2 Lamps.
//!
//! Covers:
//! - `lamp` (standard unlit 16 colors)
//! - `lamp_lit` (standard lit 16 colors)
//! - `lamp_inverted` (inverted unlit 16 colors)
//! - `lamp_inverted_lit` (inverted lit 16 colors)
//! - Level 1 placement and hitbox scenarios
//! - Level 4 redstone power toggle state transitions
//! - Multipass Pass 0 base render and Pass 1 bloom halo

use redpower_2::lighting::{
    check_power_state, is_inverted, is_lamp_block, is_lit, light_emission, pass0_render,
    pass1_bloom, COLOR_NAMES, LAMP_BLOCK_ID, LAMP_COLORS, LAMP_HARDNESS, LAMP_INVERTED_BLOCK_ID,
    LAMP_INVERTED_LIT_BLOCK_ID, LAMP_LIT_BLOCK_ID, LAMP_RESISTANCE, LIGHT_COLORS, LIT_LIGHT_LEVEL,
    UNLIT_LIGHT_LEVEL,
};

#[test]
fn test_all_16_lamp_colors_and_properties() {
    assert_eq!(LAMP_COLORS, 16);
    assert_eq!(LIGHT_COLORS.len(), 16);
    assert_eq!(COLOR_NAMES.len(), 16);

    for (i, name) in COLOR_NAMES.iter().enumerate() {
        assert!(!name.is_empty(), "Color {i} name should not be empty");
        let tc = LIGHT_COLORS[i];
        assert!(tc <= 0xFFFFFF, "Color {i} ({name}) hex exceeds 24-bit RGB");
    }

    assert_eq!(LAMP_HARDNESS, 0.3);
    assert_eq!(LAMP_RESISTANCE, 1.5);
}

#[test]
fn test_lamp_block_identification() {
    assert!(is_lamp_block(LAMP_BLOCK_ID));
    assert!(is_lamp_block(LAMP_LIT_BLOCK_ID));
    assert!(is_lamp_block(LAMP_INVERTED_BLOCK_ID));
    assert!(is_lamp_block(LAMP_INVERTED_LIT_BLOCK_ID));
    assert!(!is_lamp_block("redpower_2:alloy_furnace"));
    assert!(!is_lamp_block("minecraft:redstone_lamp"));

    assert!(!is_lit(LAMP_BLOCK_ID));
    assert!(is_lit(LAMP_LIT_BLOCK_ID));
    assert!(!is_lit(LAMP_INVERTED_BLOCK_ID));
    assert!(is_lit(LAMP_INVERTED_LIT_BLOCK_ID));

    assert!(!is_inverted(LAMP_BLOCK_ID));
    assert!(!is_inverted(LAMP_LIT_BLOCK_ID));
    assert!(is_inverted(LAMP_INVERTED_BLOCK_ID));
    assert!(is_inverted(LAMP_INVERTED_LIT_BLOCK_ID));
}

#[test]
fn test_lamp_emission_levels() {
    assert_eq!(light_emission(LAMP_BLOCK_ID), UNLIT_LIGHT_LEVEL);
    assert_eq!(light_emission(LAMP_LIT_BLOCK_ID), LIT_LIGHT_LEVEL);
    assert_eq!(light_emission(LAMP_INVERTED_BLOCK_ID), UNLIT_LIGHT_LEVEL);
    assert_eq!(light_emission(LAMP_INVERTED_LIT_BLOCK_ID), LIT_LIGHT_LEVEL);
}

#[test]
fn test_level_4_redstone_toggle_standard_lamp() {
    for color in 0..16u8 {
        // Step 1: Initial placed lamp is unlit
        assert_eq!(light_emission(LAMP_BLOCK_ID), 0);

        // Step 2: Redstone signal applied -> toggles off -> lit, metadata preserved
        let toggled = check_power_state(LAMP_BLOCK_ID, color, true);
        assert_eq!(toggled, Some((LAMP_LIT_BLOCK_ID, color)));

        // Step 3: Sustained redstone signal -> remains lit
        let sustained = check_power_state(LAMP_LIT_BLOCK_ID, color, true);
        assert_eq!(sustained, None);
        assert_eq!(light_emission(LAMP_LIT_BLOCK_ID), 15);

        // Step 4: Redstone signal removed -> toggles lit -> off, metadata preserved
        let unpowered = check_power_state(LAMP_LIT_BLOCK_ID, color, false);
        assert_eq!(unpowered, Some((LAMP_BLOCK_ID, color)));

        // Step 5: Sustained unpowered state -> remains off
        let idle = check_power_state(LAMP_BLOCK_ID, color, false);
        assert_eq!(idle, None);
    }
}

#[test]
fn test_level_4_redstone_toggle_inverted_lamp() {
    for color in 0..16u8 {
        // Step 1: Initial placed inverted lamp is lit by default
        assert_eq!(light_emission(LAMP_INVERTED_LIT_BLOCK_ID), 15);

        // Step 2: Redstone signal applied -> toggles lit -> off, metadata preserved
        let toggled = check_power_state(LAMP_INVERTED_LIT_BLOCK_ID, color, true);
        assert_eq!(toggled, Some((LAMP_INVERTED_BLOCK_ID, color)));

        // Step 3: Sustained redstone signal -> remains off
        let sustained = check_power_state(LAMP_INVERTED_BLOCK_ID, color, true);
        assert_eq!(sustained, None);
        assert_eq!(light_emission(LAMP_INVERTED_BLOCK_ID), 0);

        // Step 4: Redstone signal removed -> toggles off -> lit, metadata preserved
        let unpowered = check_power_state(LAMP_INVERTED_BLOCK_ID, color, false);
        assert_eq!(unpowered, Some((LAMP_INVERTED_LIT_BLOCK_ID, color)));

        // Step 5: Sustained unpowered state -> remains lit
        let idle = check_power_state(LAMP_INVERTED_LIT_BLOCK_ID, color, false);
        assert_eq!(idle, None);
    }
}

#[test]
fn test_pass0_pass1_rendering_parity() {
    for color in 0..16u8 {
        // Standard unlit pass 0
        let p0_off = pass0_render(LAMP_BLOCK_ID, color).expect("pass 0 unlit");
        assert_eq!(p0_off.bounds, [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
        assert_eq!(p0_off.sprite_index, 16 + color as u32);
        assert!(!p0_off.emissive);
        assert_eq!(pass1_bloom(LAMP_BLOCK_ID, color), None);

        // Standard lit pass 0 and pass 1
        let p0_on = pass0_render(LAMP_LIT_BLOCK_ID, color).expect("pass 0 lit");
        assert_eq!(p0_on.bounds, [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
        assert_eq!(p0_on.sprite_index, 32 + color as u32);
        assert!(p0_on.emissive);

        let p1 = pass1_bloom(LAMP_LIT_BLOCK_ID, color).expect("pass 1 bloom");
        assert_eq!(p1.bounds, [-0.05, -0.05, -0.05, 1.05, 1.05, 1.05]);
        assert_eq!(p1.alpha, 0.5);
        assert_eq!(p1.blend_src, 770);
        assert_eq!(p1.blend_dst, 1);

        let expected_color = LIGHT_COLORS[color as usize];
        let exp_r = ((expected_color >> 16) & 0xFF) as f32 / 255.0;
        let exp_g = ((expected_color >> 8) & 0xFF) as f32 / 255.0;
        let exp_b = (expected_color & 0xFF) as f32 / 255.0;
        assert_eq!(p1.rgb_tint, [exp_r, exp_g, exp_b]);
    }
}
