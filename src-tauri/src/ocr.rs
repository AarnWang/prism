use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrRequest {
    pub image_data: Vec<u8>,
    pub image_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub text: String,
    pub confidence: f32,
}

pub struct OcrService {
    api_key: String,
    base_url: String,
    model_id: String,
}

impl OcrService {
    pub fn new(api_key: String, base_url: String, model_id: String) -> Self {
        Self {
            api_key,
            base_url,
            model_id,
        }
    }

    pub async fn extract_text(&self, request: OcrRequest) -> Result<OcrResult, String> {
        // 将图片转换为base64
        let base64_image = general_purpose::STANDARD.encode(&request.image_data);
        let data_url = format!(
            "data:image/{};base64,{}",
            request.image_format, base64_image
        );

        let client = reqwest::Client::new();
        println!(
            "正在发送OCR请求...{}, 模型:{}",
            self.base_url, self.model_id
        );
        let body = serde_json::json!({
            "model": self.model_id,
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": "Please extract all the text content from the image, only return the recognized text, without adding any explanation or formatting. If there is no text in the image, please return an empty string."
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": data_url
                            }
                        }
                    ]
                }
            ],
            "max_tokens": 1000,
            "temperature": 0.1
        });

        let endpoint = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let response = client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("发送OCR请求失败: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("OCR API错误: {}", error_text));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析OCR响应失败: {}", e))?;

        let extracted_text = response_json
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        println!("orc解析文本: {}", extracted_text);
        Ok(OcrResult {
            text: extracted_text,
            confidence: 0.95,
        })
    }
}
