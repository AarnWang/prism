use crate::{
    app_state::AppState,
    ocr::{OcrRequest, OcrService},
};
use tauri::State;

pub async fn run_ocr_on_image_data(
    image_data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let (api_key, base_url, model_id) = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("获取数据库连接失败: {}", e))?;

        match db.get_app_config() {
            Ok(Some(config)) => {
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
                    )
                } else {
                    let ocr_config = config.ocr;
                    (ocr_config.api_key, ocr_config.base_url, ocr_config.model_id)
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
    let ocr_request = OcrRequest {
        image_data,
        image_format: "png".to_string(),
    };

    let ocr_result = ocr_service
        .extract_text(ocr_request)
        .await
        .map_err(|e| format!("OCR识别失败: {}", e))?;

    Ok(ocr_result.text)
}
