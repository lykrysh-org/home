use actix_web::error::Result;
use actix_web::middleware::session::RequestSession;
use actix_web::HttpRequest;

const UPLOADED: &str = "uploaded";
const CHATCNT: &str = "chatcnt";

pub fn set_uploaded<T>(req: &HttpRequest<T>, uploaded: UpLoaded) -> Result<()> {
    req.session().set(UPLOADED, uploaded)
}

pub fn get_uploaded<T>(req: &HttpRequest<T>) -> Result<Option<UpLoaded>> {
    req.session().get::<UpLoaded>(UPLOADED)
}

pub fn clear_uploaded<T>(req: &HttpRequest<T>) {
    req.session().remove(UPLOADED);
}

#[derive(Deserialize, Serialize)]
pub struct UpLoaded {
    pub uploaded: String,
}

impl UpLoaded {
    pub fn add(uploaded: &str) -> Self {
        Self { uploaded: uploaded.to_owned(), }
    }
}

pub fn set_ccnt<T>(req: &HttpRequest<T>, counter: ChatCnt) -> Result<()> {
    req.session().set(CHATCNT, counter)
}

pub fn get_ccnt<T>(req: &HttpRequest<T>) -> Result<Option<ChatCnt>> {
    req.session().get::<ChatCnt>(CHATCNT)
}

pub fn clear_ccnt<T>(req: &HttpRequest<T>) {
    req.session().remove(CHATCNT);
}

#[derive(Deserialize, Serialize)]
pub struct ChatCnt {
    pub i: i64,
}

impl ChatCnt {
    pub fn init() -> Self {
        Self { i: 0, }
    }
    pub fn change(cnt: i64) -> Self {
        Self { i: cnt, }
    }
}
