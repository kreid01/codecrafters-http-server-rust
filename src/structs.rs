#[derive(Debug, Clone)]
pub struct HTTPRequest {
    pub method: HTTPMethod,
    pub target: String,
    pub version: String,
    pub content_type: String,
    pub headers: HTTPHeaders,
    pub body: String,
    pub content: String,
    pub encoding: Option<String>,
}

#[derive(Debug, Clone)]
pub enum HTTPMethod {
    Get,
    Post,
}

#[derive(Debug, Clone)]
pub struct HTTPHeaders {
    pub host: String,
    pub user_agent: String,
    // accept: String,
}
