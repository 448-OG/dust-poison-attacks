use dioxus::prelude::*;
use wasm_bindgen_futures::{wasm_bindgen::JsCast, JsFuture};
use web_sys::{wasm_bindgen::JsValue, Headers, Request, RequestInit, RequestMode, Response};

use crate::{utils::ENDPOINT, GLOBAL_MESSAGE};

pub type JsResult<T> = Result<T, JsValue>;

#[derive(Debug)]
pub struct FetchReq {
    headers: Headers,
    options: RequestInit,
}

impl FetchReq {
    pub fn new(method: &str) -> JsResult<Self> {
        let options = RequestInit::new();
        options.set_method(method);
        options.set_mode(RequestMode::Cors);

        Ok(Self {
            headers: Headers::new()?,
            options,
        })
    }

    pub fn new_for_rpc() -> JsResult<Self> {
        Self::new("POST")?.add_header("Content-Type", "application/json")
    }

    pub fn add_header(self, key: &str, value: &str) -> JsResult<Self> {
        self.headers.append(key, value)?;

        Ok(self)
    }

    pub fn set_body(self, json_body: &str) -> Self {
        self.options.set_body(&json_body.into());

        self
    }

    pub async fn send(self, tx: &str) -> JsResult<String> {
        let resp = self.build(tx).await?;

        JsFuture::from(resp.text()?)
            .await?
            .as_string()
            .ok_or("The response body is not a JsString".into())
    }

    pub async fn build(&self, tx: &str) -> JsResult<Response> {
        self.options.set_headers(&self.headers);

        let address = String::from("") + "/tx/" + tx;

        let request = Request::new_with_str_and_init(&address, &self.options)?;

        let fetch_promise = web_sys::window().unwrap().fetch_with_request(&request);

        // Await the fetch promise to get a `Response` object
        let resp_value = JsFuture::from(fetch_promise).await?;
        Ok(resp_value.dyn_into::<Response>()?)
    }
}

#[derive(Debug, Clone)]
pub struct NotificationInfo {
    key: u32,
    secs: u32,
    message: String,
}

impl NotificationInfo {
    pub fn new(message: impl core::fmt::Display) -> Self {
        let key = fastrand::u32(..);

        Self {
            key,
            secs: 2,
            message: message.to_string(),
        }
    }

    /// Sets default seconds to 15
    pub fn error(message: impl core::fmt::Display) -> Self {
        Self::new(message).set_secs(15)
    }

    pub fn set_secs(mut self, secs: u32) -> Self {
        self.secs = secs;

        self
    }

    pub fn key(&self) -> u32 {
        self.key
    }

    pub fn secs(&self) -> u32 {
        self.secs
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}
