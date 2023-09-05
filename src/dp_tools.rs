use std::collections::HashMap;

use serde_json::Value;

const PLAY_AND_GET_DIGITS_APP: &str = "play_and_get_digits";
const PLAYBACK_APP: &str = "playback";

use crate::{EslConnection, EslError, Event};

impl EslConnection {
    /// plays file in call during outbound mode
    pub async fn playback(&self, file_path: &str) -> Result<Event, EslError> {
        self.execute(PLAYBACK_APP, file_path).await
    }
 
    /// record_session during outbound mode
    pub async fn record_session(&self, file_path: &str) -> Result<Event, EslError> {
        self.execute("record_session", file_path).await
    }
 
    /// send dtmf during outbound mode
    pub async fn send_dtmf(&self, dtmf_str: &str) -> Result<Event, EslError> {
        self.execute("send_dtmf", dtmf_str).await
    }

    /// wait for silence during outbound mode
    pub async fn wait_for_silence(&self, silence_str: &str) -> Result<Event, EslError> {
        self.execute("wait_for_silence", silence_str).await
    }

    /// sleep for specified milliseconds in outbound mode
    pub async fn sleep(&self, millis: i128) -> Result<Event, EslError> {
        self.execute("sleep", &millis.to_string()).await
    }

    ///set a channel variable 
    pub async fn set_variable(&self, var: &str, value: &str) -> Result<Event, EslError> {
        let args = format!("{}={}", var, value);
        self.execute("set", &args).await
    }
 
    ///add  a freeswitch log
    pub async fn fs_log(&self, loglevel: &str, msg: &str) -> Result<Event, EslError> {
        let args = format!("{} {}", loglevel, msg);
        self.execute("log", &args).await
    }
    

    #[allow(clippy::too_many_arguments)]
    /// Used for mod_play_and_get_digits
    pub async fn play_and_get_digits(
        &self,
        min: u8,
        max: u8,
        tries: u8,
        timeout: u64,
        terminators: &str,
        file: &str,
        invalid_file: &str,
    ) -> Result<String, EslError> {
        let variable_name = uuid::Uuid::new_v4().to_string();
        let app_args = format!(
            "{min} {max} {tries} {timeout} {terminators} {file} {invalid_file} {variable_name}",
        );
        let data = self.execute(PLAY_AND_GET_DIGITS_APP, &app_args).await?;
        let body = data.body.as_ref().unwrap();
        let body = parse_json_body(body).unwrap();
        let result = body.get(&format!("variable_{}", variable_name));
        let Some(digit) = result else {
            return Err(EslError::NoInput);
        };
        let digit = digit.as_str().unwrap().to_string();
        Ok(digit)
    }
}

fn parse_json_body(body: &str) -> Result<HashMap<String, Value>, EslError> {
    Ok(serde_json::from_str(body)?)
}
