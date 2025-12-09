use crate::models::{ASRConfig, EchoKitConfig, TTSConfig};

/// 生成 ASR 配置部分
fn generate_asr_config(asr: &ASRConfig) -> String {
    match asr {
        ASRConfig::Openai {
            api_key,
            model,
            lang,
            prompt,
            url,
        } => {
            let url = url
                .as_deref()
                .unwrap_or("https://api.openai.com/v1/audio/transcriptions");
            let prompt_value = prompt
                .as_deref()
                .unwrap_or("Hello\n你好\n(noise)\n(bgm)\n(silence)\n");
            format!(
                r#"[asr]
url = "{url}"
api_key = "{api_key}"
model = "{model}"
lang = "{lang}"
prompt = """
{prompt_value}"""
vad_url = "http://host.docker.internal:8000/v1/audio/vad"
"#
            )
        }
        ASRConfig::Paraformer { paraformer_token } => {
            format!(
                r#"[asr]
paraformer_token = "{paraformer_token}"
"#
            )
        }
    }
}

/// 生成 TTS 配置部分
fn generate_tts_config(tts: &TTSConfig) -> String {
    match tts {
        TTSConfig::Openai {
            api_key,
            model,
            voice,
            url,
        } => {
            let url = url
                .as_deref()
                .unwrap_or("https://api.openai.com/v1/audio/speech");
            format!(
                r#"[tts]
platform = "Openai"
api_key = "{api_key}"
model = "{model}"
voice = "{voice}"
url = "{url}"
"#
            )
        }
        TTSConfig::Groq {
            api_key,
            model,
            voice,
            url,
        } => {
            let url = url
                .as_deref()
                .unwrap_or("https://api.groq.com/openai/v1/audio/speech");
            format!(
                r#"[tts]
platform = "Groq"
api_key = "{api_key}"
model = "{model}"
voice = "{voice}"
url = "{url}"
"#
            )
        }
        TTSConfig::Elevenlabs {
            token,
            voice,
            model_id,
            language_code,
        } => {
            let model_id_line = match model_id {
                Some(m) => format!("model_id = \"{m}\"\n"),
                None => String::new(),
            };
            let language_code_line = match language_code {
                Some(l) => format!("language_code = \"{l}\"\n"),
                None => String::new(),
            };
            format!(
                r#"[tts]
platform = "Elevenlabs"
token = "{token}"
voice = "{voice}"
{model_id_line}{language_code_line}"#
            )
        }
        TTSConfig::GSV {
            url,
            speaker,
            api_key,
            timeout_sec,
        } => {
            let api_key_line = match api_key {
                Some(k) => format!("api_key = \"{k}\"\n"),
                None => String::new(),
            };
            let timeout_line = match timeout_sec {
                Some(t) => format!("timeout_sec = {t}\n"),
                None => String::new(),
            };
            format!(
                r#"[tts]
platform = "GSV"
url = "{url}"
speaker = "{speaker}"
{api_key_line}{timeout_line}"#
            )
        }
        TTSConfig::StreamGSV {
            url,
            speaker,
            api_key,
        } => {
            let api_key_line = match api_key {
                Some(k) => format!("api_key = \"{k}\"\n"),
                None => String::new(),
            };
            format!(
                r#"[tts]
platform = "StreamGSV"
url = "{url}"
speaker = "{speaker}"
{api_key_line}"#
            )
        }
        TTSConfig::Fish { api_key, speaker } => {
            format!(
                r#"[tts]
platform = "Fish"
api_key = "{api_key}"
speaker = "{speaker}"
"#
            )
        }
        TTSConfig::CosyVoice {
            token,
            speaker,
            version,
        } => {
            let speaker_line = match speaker {
                Some(s) => format!("speaker = \"{s}\"\n"),
                None => String::new(),
            };
            let version_line = match version {
                Some(v) => format!("version = \"{v}\"\n"),
                None => String::new(),
            };
            format!(
                r#"[tts]
platform = "CosyVoice"
token = "{token}"
{speaker_line}{version_line}"#
            )
        }
    }
}

/// 生成 EchoKit Server 的 config.toml 内容
pub fn generate_config_toml(config: &EchoKitConfig) -> String {
    let llm_history = config.llm.history.unwrap_or(5);
    let tts_config = generate_tts_config(&config.tts);
    let asr_config = generate_asr_config(&config.asr);

    format!(
        r#"addr = "0.0.0.0:8080"
hello_wav = "hello.wav"

{tts_config}
{asr_config}
[llm]
llm_chat_url = "{llm_url}"
api_key = "{llm_key}"
model = "{llm_model}"
history = {llm_history}

[[llm.sys_prompts]]
role = "system"
content = """
{llm_prompt}
"""
"#,
        tts_config = tts_config,
        asr_config = asr_config,
        llm_url = config.llm.url,
        llm_key = config.llm.api_key,
        llm_model = config.llm.model,
        llm_history = llm_history,
        llm_prompt = config.llm.system_prompt,
    )
}
