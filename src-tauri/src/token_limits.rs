use crate::database::TokenLimitConfig;

pub const MIN_AI_MAX_TOKENS: u32 = 1000;

fn clamp_with_limits(estimated: u64, config: Option<&TokenLimitConfig>) -> u32 {
    let mut value = estimated.max(MIN_AI_MAX_TOKENS as u64);

    if let Some(cfg) = config {
        if cfg.enable_user_max_tokens {
            let user_max = (cfg.user_max_tokens as u64).max(MIN_AI_MAX_TOKENS as u64);
            value = value.min(user_max);
        }
    }

    value.min(u32::MAX as u64) as u32
}

pub fn calculate_text_response_tokens(
    text: &str,
    config: Option<&TokenLimitConfig>,
) -> u32 {
    if text.is_empty() {
        return clamp_with_limits(MIN_AI_MAX_TOKENS as u64, config);
    }

    let char_count = text.chars().count() as u64;
    let scaled = ((char_count as f64) * 1.2).ceil() as u64;
    let buffered = scaled.saturating_add(256);

    clamp_with_limits(buffered, config)
}

pub fn calculate_image_response_tokens(
    width: u32,
    height: u32,
    config: Option<&TokenLimitConfig>,
) -> u32 {
    if width == 0 || height == 0 {
        return clamp_with_limits(MIN_AI_MAX_TOKENS as u64, config);
    }

    let area = width as u64 * height as u64;
    // Assume each readable character roughly occupies a 350px^2 area and add buffer
    let approx_chars = ((area as f64) / 350.0).ceil() as u64;
    let buffered = approx_chars.saturating_add(512);

    clamp_with_limits(buffered, config)
}
