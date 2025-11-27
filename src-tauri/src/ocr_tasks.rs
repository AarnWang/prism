use crate::{
    app_state::AppState,
    ocr::{OcrRequest, OcrService},
    token_limits::calculate_image_response_tokens,
};
use image::{imageops::FilterType, GenericImageView, ImageFormat};
use std::io::Cursor;
use tauri::State;

const MIN_OCR_DIMENSION: u32 = 28;

fn ensure_minimum_ocr_size(image_data: Vec<u8>) -> Result<(Vec<u8>, u32, u32), String> {
    let image =
        image::load_from_memory(&image_data).map_err(|e| format!("解析OCR图片失败: {}", e))?;
    let (width, height) = image.dimensions();

    if width == 0 || height == 0 {
        return Err("截图区域太小，无法进行OCR".to_string());
    }

    if width >= MIN_OCR_DIMENSION && height >= MIN_OCR_DIMENSION {
        return Ok((image_data, width, height));
    }

    let scale = f32::max(
        MIN_OCR_DIMENSION as f32 / width as f32,
        MIN_OCR_DIMENSION as f32 / height as f32,
    );

    let target_width = (width as f32 * scale).ceil() as u32;
    let target_height = (height as f32 * scale).ceil() as u32;
    let resized = image.resize_exact(target_width, target_height, FilterType::CatmullRom);

    let mut buffer = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
        .map_err(|e| format!("编码OCR图片失败: {}", e))?;

    println!(
        "OCR截图尺寸过小 ({}x{}), 已缩放至 {}x{}",
        width, height, target_width, target_height
    );

    Ok((buffer, target_width, target_height))
}

pub async fn run_ocr_on_image_data(
    image_data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let (processed_image_data, width, height) = ensure_minimum_ocr_size(image_data)?;

    let (api_key, base_url, model_id, token_config) = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("获取数据库连接失败: {}", e))?;

        match db.get_app_config() {
            Ok(Some(config)) => {
                let token_config = config.token_limits.clone();
                if config.ocr.reuse_translation {
                    let translation_config = config.translation;
                    let model_id = if !config.ocr.model_id.is_empty() {
                        config.ocr.model_id
                    } else {
                        translation_config.model_id
                    };
                    (
                        translation_config.api_key,
                        translation_config.base_url,
                        model_id,
                        token_config,
                    )
                } else {
                    let ocr_config = config.ocr;
                    (
                        ocr_config.api_key,
                        ocr_config.base_url,
                        ocr_config.model_id,
                        token_config,
                    )
                }
            }
            Ok(None) => {
                return Err("OCR配置不存在，请先在设置中配置OCR".to_string());
            }
            Err(e) => {
                return Err(format!("获取OCR配置失败: {}", e));
            }
        }
    };

    if api_key.is_empty() {
        return Err("OCR API密钥未配置，请在设置中配置API密钥".to_string());
    }

    let ocr_service = OcrService::new(api_key, base_url, model_id);
    let max_tokens =
        calculate_image_response_tokens(width, height, Some(&token_config));
    let ocr_request = OcrRequest {
        image_data: processed_image_data,
        image_format: "png".to_string(),
        max_tokens,
    };

    let ocr_result = ocr_service
        .extract_text(ocr_request)
        .await
        .map_err(|e| format!("OCR识别失败: {}", e))?;

    Ok(ocr_result.text)
}
