#[cfg(test)]
mod tests {
    use capybar::util::Color;

    #[test]
    fn test_from_rgba() {
        let c = Color::from_rgba(0x12, 0x34, 0x56, 0x78);
        assert_eq!(c.r(), 0x12);
        assert_eq!(c.g(), 0x34);
        assert_eq!(c.b(), 0x56);
        assert_eq!(c.a(), 0x78);
    }

    #[test]
    fn test_from_hex() {
        let c = Color::from_hex(0x12345678);
        assert_eq!(c.r(), 0x12);
        assert_eq!(c.g(), 0x34);
        assert_eq!(c.b(), 0x56);
        assert_eq!(c.a(), 0x78);
    }

    #[test]
    fn test_from_be_bytes() {
        let bytes = [0x12, 0x34, 0x56, 0x78];
        let c = Color::from_be_bytes(&bytes);
        assert_eq!(c.to_be_bytes(), bytes);
    }

    #[test]
    fn test_from_le_bytes() {
        let bytes = [0x78, 0x56, 0x34, 0x12];
        let c = Color::from_le_bytes(&bytes);
        assert_eq!(c.to_le_bytes(), bytes);
    }

    #[test]
    fn test_from_rgba_f32_valid() {
        let c = Color::from_rgba_f32(0.0, 0.5, 1.0, 0.0).unwrap();
        assert_eq!(c.r(), 0);
        assert_eq!(c.g(), 128);
        assert_eq!(c.b(), 255);
        assert_eq!(c.a(), 0);
    }

    #[test]
    fn test_from_rgba_f32_out_of_range() {
        assert!(Color::from_rgba_f32(-0.1, 0.0, 0.0, 0.0).is_none());
        assert!(Color::from_rgba_f32(1.1, 0.0, 0.0, 0.0).is_none());
        assert!(Color::from_rgba_f32(0.0, -1.0, 0.0, 0.0).is_none());
        assert!(Color::from_rgba_f32(0.0, 0.0, 2.0, 0.0).is_none());
        assert!(Color::from_rgba_f32(0.0, 0.0, 0.0, -0.5).is_none());
        assert!(Color::from_rgba_f32(0.0, 0.0, 0.0, 1.5).is_none());
    }

    #[test]
    fn test_setters() {
        let mut c = Color::from_rgba(0, 0, 0, 0);

        c.set_r(0x12);
        assert_eq!(c.r(), 0x12);
        assert_eq!(c.g(), 0);
        assert_eq!(c.b(), 0);
        assert_eq!(c.a(), 0);

        c.set_g(0x34);
        assert_eq!(c.r(), 0x12);
        assert_eq!(c.g(), 0x34);
        assert_eq!(c.b(), 0);
        assert_eq!(c.a(), 0);

        c.set_b(0x56);
        assert_eq!(c.r(), 0x12);
        assert_eq!(c.g(), 0x34);
        assert_eq!(c.b(), 0x56);
        assert_eq!(c.a(), 0);

        c.set_a(0x78);
        assert_eq!(c.r(), 0x12);
        assert_eq!(c.g(), 0x34);
        assert_eq!(c.b(), 0x56);
        assert_eq!(c.a(), 0x78);
    }

    #[test]
    fn test_blending_edge_cases() {
        let bg = Color::from_rgba(100, 150, 200, 255);
        let fg = Color::from_rgba(0, 0, 0, 0);
        assert_eq!(Color::blend_colors(&bg, &fg), bg);

        let fg = Color::from_rgba(50, 100, 150, 255);
        assert_eq!(Color::blend_colors(&bg, &fg), fg);

        let bg = Color::from_rgba(0, 0, 0, 0);
        let fg = Color::from_rgba(75, 125, 175, 128);
        assert_eq!(Color::blend_colors(&bg, &fg), fg);

        let fg = Color::from_rgba(0, 0, 0, 0);
        assert_eq!(Color::blend_colors(&bg, &fg), fg);
    }

    #[test]
    fn test_blending_accuracy() {
        let white = Color::from_rgba(255, 255, 255, 255);
        let gray = Color::from_rgba(128, 128, 128, 128);
        let blended = Color::blend_colors(&white, &gray);
        assert_eq!(blended, Color::from_rgba(191, 191, 191, 255));

        let blue = Color::from_rgba(0, 0, 255, 255);
        let red = Color::from_rgba(255, 0, 0, 128);
        let blended = Color::blend_colors(&blue, &red);
        assert_eq!(blended, Color::from_rgba(128, 0, 127, 255));

        let bg = Color::from_rgba(100, 100, 100, 128);
        let fg = Color::from_rgba(200, 200, 200, 128);
        let blended = Color::blend_colors(&bg, &fg);
        assert_eq!(blended, Color::from_rgba(167, 167, 167, 191));
    }

    #[test]
    fn test_blend_semi_transparent() {
        let bg = Color::from_rgba(100, 100, 100, 255);
        let fg = Color::from_rgba(200, 200, 200, 128);
        let blended = Color::blend_colors(&bg, &fg);

        assert!(blended.r() > 100 && blended.r() < 200);
        assert!(blended.g() > 100 && blended.g() < 200);
        assert!(blended.b() > 100 && blended.b() < 200);
    }

    #[test]
    fn test_blending_alpha_boundaries() {
        let bg = Color::from_rgba(0, 0, 0, 255);
        let fg = Color::from_rgba(255, 255, 255, 1);
        let blended = Color::blend_colors(&bg, &fg);
        assert_eq!(blended.a(), 255);

        let fg = Color::from_rgba(255, 0, 0, 254);
        let blended = Color::blend_colors(&bg, &fg);
        assert_eq!(blended, Color::from_rgba(254, 0, 0, 255));
    }

    #[test]
    fn test_blending_extreme_values() {
        let white = Color::from_rgba(255, 255, 255, 255);
        let black = Color::from_rgba(0, 0, 0, 255);
        assert_eq!(Color::blend_colors(&white, &black), black);

        let transparent = Color::from_rgba(0, 0, 0, 0);
        let visible = Color::from_rgba(10, 20, 30, 40);
        assert_eq!(Color::blend_colors(&transparent, &visible), visible);

        let bg = Color::from_rgba(255, 255, 255, 255);
        let fg = Color::from_rgba(255, 255, 255, 255);
        let blended = Color::blend_colors(&bg, &fg);
        assert_eq!(blended, Color::from_rgba(255, 255, 255, 255));
    }
    #[test]
    fn test_no_overflow() {
        let c = Color::from_rgba(255, 255, 255, 255);
        assert_eq!(c.r(), 255);
        assert_eq!(c.g(), 255);
        assert_eq!(c.b(), 255);
        assert_eq!(c.a(), 255);

        let c = Color::from_rgba(0, 0, 0, 0);
        assert_eq!(c.r(), 0);
        assert_eq!(c.g(), 0);
        assert_eq!(c.b(), 0);
        assert_eq!(c.a(), 0);

        let bg = Color::from_rgba(255, 255, 255, 255);
        let fg = Color::from_rgba(0, 0, 0, 0);
        let blended = Color::blend_colors(&bg, &fg);
        assert_eq!(blended, bg);
    }

    #[test]
    fn test_display() {
        assert_eq!(
            format!("{}", Color::from_rgba(0x12, 0x34, 0x56, 0x78)),
            "0x12345678"
        );
        assert_eq!(format!("{}", Color::from_rgba(0, 0, 0, 0)), "0x00000000");
        assert_eq!(
            format!("{}", Color::from_rgba(0xFF, 0xFF, 0xFF, 0xFF)),
            "0xffffffff"
        );
        assert_eq!(
            format!("{}", Color::from_rgba(0xAB, 0xCD, 0xEF, 0x12)),
            "0xabcdef12"
        );
    }

    #[test]
    fn test_byte_conversion_roundtrip() {
        let original = Color::from_rgba(0x12, 0x34, 0x56, 0x78);

        let be_bytes = original.to_be_bytes();
        assert_eq!(Color::from_be_bytes(&be_bytes), original);

        let le_bytes = original.to_le_bytes();
        assert_eq!(Color::from_le_bytes(&le_bytes), original);
    }
}
